//! [`SpatialIdMap`](crate::SpatialIdMap) のバイト列（[`super::arena`]）を、`Arc` 木を再構築せず
//! 直接読む ZeroCopy リーダ。
//!
//! [`SpatialIdMap::to_bytes`](crate::SpatialIdMap::to_bytes) が書いたバイト列は
//! [`SpatialIdMap::from_bytes`](crate::SpatialIdMap::from_bytes) で完全な作業木へ
//! 復元することもできるが、読むだけならその再構築コストは不要。[`ArchivedSpatialIdMap`] は
//! archived バイト列を**直接走査**し、`&[u8]` を ZeroCopy で返す読み取り専用の窓口を提供する。

use alloc::vec::Vec;

use super::arena::{ArchivedArenaNode, ArchivedMapArena, EMPTY_LEAF, check_version};
use crate::spatial_id::collection::flex_tree::core::walk::{
    OverlapWalk, RangeOverlapWalk, TreeCursor,
};
use crate::{Error, FlexId};

/// バイト列を直接走査する ZeroCopy リーダ。`Arc` 木を再構築しない。
///
/// [`SpatialIdMap::to_bytes`](crate::SpatialIdMap::to_bytes) が生成したバイト列に対する、
/// 読み取り専用の対をなす型。書き込みが必要なら
/// [`SpatialIdMap::from_bytes`](crate::SpatialIdMap::from_bytes) で作業木へ復元すること。
pub struct ArchivedSpatialIdMap<'a> {
    inner: &'a ArchivedMapArena,
}

/// archived アリーナ上の1ノードを指すカーソル。
///
/// `TreeCursor` の実装対象。`Copy` なのでスタックへ安価に積める。
///
/// ノード参照を**持ち回る**のが要点。インデックスだけを持つと `branch()` と
/// `leaf_value()` で同じ要素を2回引くことになり、葉の数だけ余計な境界検査が入る
/// （実測で範囲走査が +15〜23%）。
#[derive(Clone, Copy)]
pub(crate) struct ArchivedCursor<'a> {
    nodes: &'a rkyv::vec::ArchivedVec<ArchivedArenaNode>,
    node: &'a ArchivedArenaNode,
}

impl<'a> ArchivedCursor<'a> {
    fn at(nodes: &'a rkyv::vec::ArchivedVec<ArchivedArenaNode>, idx: u32) -> Self {
        Self {
            nodes,
            node: &nodes[idx as usize],
        }
    }

    /// 値付き葉なら辞書インデックス（1始まり）。分岐・空葉なら `None`。
    fn leaf_value(self) -> Option<u32> {
        match self.node {
            ArchivedArenaNode::Leaf { value } => match value.to_native() {
                EMPTY_LEAF => None,
                v => Some(v),
            },
            ArchivedArenaNode::Branch { .. } => None,
        }
    }
}

impl<'a> TreeCursor for ArchivedCursor<'a> {
    fn branch(self) -> Option<(u8, Self, Self)> {
        match self.node {
            ArchivedArenaNode::Branch {
                level,
                lower,
                upper,
            } => Some((
                *level,
                Self::at(self.nodes, lower.to_native()),
                Self::at(self.nodes, upper.to_native()),
            )),
            ArchivedArenaNode::Leaf { .. } => None,
        }
    }
}

impl<'a> ArchivedSpatialIdMap<'a> {
    /// アリーナ上の `idx` を指すカーソルを作る。
    fn cursor(&self, idx: u32) -> ArchivedCursor<'a> {
        ArchivedCursor::at(&self.inner.nodes, idx)
    }

    /// archived バイト列上にリーダを開く。
    ///
    /// 形式バージョンだけは検証する（`u16` の読み出しと比較1回なので、
    /// リーフごとに呼ばれる読み取りホットパスでも無視できるコスト）。
    /// バイト列全体の構造検証は行わない。
    ///
    /// # Safety
    /// `bytes` は [`crate::SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn access(bytes: &'a [u8]) -> Result<Self, Error> {
        let inner = unsafe { rkyv::access_unchecked::<ArchivedMapArena>(bytes) };
        check_version(inner.version.to_native())?;
        Ok(Self { inner })
    }

    /// このバイト列に書かれている形式バージョン。
    pub fn format_version(&self) -> u16 {
        self.inner.version.to_native()
    }

    /// `target` と重なるセルを走査し、各セルごとに `visit(clipped_id, packed_value)` を呼ぶ。
    ///
    /// `packed_value` は**この葉ローカルの辞書インデックス（1始まり）**で、[`value_bytes`](Self::value_bytes)
    /// で実バイト列へ復元できる。中間 `Vec` を作らないため、大量セルの集約（値ごとのグルーピング）を
    /// バイト列ではなく整数キーで行えるようにするための低レベル API。
    ///
    /// 葉は `target` で**切り取って**返す（検索が要求するのは要求範囲内のセルのため）。
    pub fn get_indexed(&self, target: &FlexId, mut visit: impl FnMut(FlexId, u32)) {
        // F はズーム0で2セルしかないので、符号が属する側のルートだけを降りればよい。
        let root = if target.f_index().is_negative() {
            (
                self.cursor(self.inner.lower_root.to_native()),
                FlexId::LOWER_MAX,
            )
        } else {
            (
                self.cursor(self.inner.upper_root.to_native()),
                FlexId::UPPER_MAX,
            )
        };

        for (current_id, leaf) in OverlapWalk::new(alloc::vec![root], *target) {
            if let Some(packed) = leaf.leaf_value()
                && let Some(clipped) = current_id.intersection(target)
            {
                visit(clipped, packed);
            }
        }
    }

    /// [`get_indexed`](Self::get_indexed) が渡す辞書インデックス（1始まり）から実バイト列を引く。
    pub fn value_bytes(&self, packed: u32) -> &'a [u8] {
        self.inner.dictionary[(packed - 1) as usize].as_slice()
    }

    /// `target`（範囲）と重なる (FlexId, 値) を ZeroCopy で列挙する。
    ///
    /// インメモリ側の `FlexTreeCore::range_overlap_ref` と同じ意味論で、葉は
    /// **切り取らずに**そのまま返す（クエリの入力源はセル全体の値を必要とするため）。
    pub fn get_range(&self, target: &crate::RangeId) -> Vec<(FlexId, &'a [u8])> {
        // F はズーム0で 0（上半球）/ -1（下半球）の2セルしか無いので、
        // 範囲を半球ごとに割ってから、該当するルートだけを降りる。
        let mut roots = Vec::new();
        if target.f()[0] < 0 {
            let mut lower_target = target.clone();
            if lower_target
                .set_f([target.f()[0], target.f()[1].min(-1)])
                .is_ok()
            {
                roots.push((
                    self.cursor(self.inner.lower_root.to_native()),
                    FlexId::LOWER_MAX,
                    lower_target,
                ));
            }
        }
        if target.f()[1] >= 0 {
            let mut upper_target = target.clone();
            if upper_target
                .set_f([target.f()[0].max(0), target.f()[1]])
                .is_ok()
            {
                roots.push((
                    self.cursor(self.inner.upper_root.to_native()),
                    FlexId::UPPER_MAX,
                    upper_target,
                ));
            }
        }

        let mut out = Vec::new();
        for (id, leaf) in RangeOverlapWalk::new(roots) {
            if let Some(packed) = leaf.leaf_value() {
                out.push((id, self.value_bytes(packed)));
            }
        }
        out
    }
}
