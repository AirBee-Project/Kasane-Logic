//! [`SpatialIdMap`] の永続化（フラットアリーナ直列化）と ZeroCopy 読み出し。
//!
//! インメモリの作業構造（`Arc` ベースの `FlexTreeCore`）はそのままに、保存時のみ
//! 木を `Vec<PersistedNode>`（子ノードは配列インデックス参照）へ平坦化して rkyv で直列化する。
//! 値は `dictionary: Vec<Vec<u8>>` に集約（重複排除）し、葉は dictionary のインデックス（+1、0 は空）を持つ。
//!
//! - 書き込み（[`SpatialIdMap::to_bytes`] / [`SpatialIdMap::from_bytes`]）は `Arc` 木との相互変換。
//! - 読み出し（[`ArchivedMap`]）は archived バイト列を**直接走査**し、`Arc` 木を再構築せず
//!   `&[u8]` を ZeroCopy で返す。

use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use rkyv::{Archive, Deserialize, Serialize};

use super::SpatialIdMap;
use crate::spatial_id::collection::flex_tree::core::node::Node;
use crate::spatial_id::collection::flex_tree::core::ptr::SharedNode;
use crate::spatial_id::collection::flex_tree::core::walk::{
    OverlapWalk, RangeOverlapWalk, TreeCursor,
};
use crate::{Error, FlexId};

/// 形式バージョンを検証する。
fn check_version(found: u16) -> Result<(), Error> {
    if found == FORMAT_VERSION {
        Ok(())
    } else {
        Err(Error::UnsupportedFormatVersion {
            expected: FORMAT_VERSION,
            found,
        })
    }
}

/// 空を表す葉インデックス値（`PersistedNode::Leaf { value }` の `value == 0`）。
const EMPTY_LEAF: u32 = 0;

/// 永続化バイト列の形式バージョン。
///
/// `PersistedMap` / `PersistedNode` のフィールド構成を変更したら**必ず上げる**。
/// 上げ忘れると、古いバイト列が拒否されずに誤って読まれる（`access_unchecked` は
/// レイアウトを検証しないため）。
///
/// なお **`FlexTreeCore` や `Node` にフィールドを足しただけならバージョンは変わらない**。
/// ディスク形式は `PersistedMap` が単独で決めており、書き込み側は `Node` の
/// キャッシュ用フィールドを読まないため。この不変性は golden テストが担保している。
pub const FORMAT_VERSION: u16 = 1;

/// 平坦化された [`SpatialIdMap`] 1枚（1シャード）。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub(crate) struct PersistedMap {
    /// 形式バージョン（[`FORMAT_VERSION`]）。読み出し時に検証する。
    version: u16,
    /// 下半分（f < 0）ルートの `nodes` インデックス。
    lower_root: u32,
    /// 上半分（f >= 0）ルートの `nodes` インデックス。
    upper_root: u32,
    /// このマップが閉じているシャード領域（挿入クリップ用）。
    shard: Option<FlexId>,
    /// 後行順（子が親より前）で並んだノード配列。
    nodes: Vec<PersistedNode>,
    /// 値の辞書。葉の `value`（>0）から `value - 1` で参照する。
    dictionary: Vec<Vec<u8>>,
}

/// 平坦化されたノード。子は `nodes` 配列のインデックス。
#[derive(Archive, Serialize, Deserialize, Debug)]
pub(crate) enum PersistedNode {
    Branch {
        level: u8,
        lower: u32,
        upper: u32,
    },
    /// `value == 0` は空、`value > 0` は `dictionary[value - 1]`。
    Leaf {
        value: u32,
    },
}

impl SpatialIdMap<Vec<u8>> {
    /// この [`SpatialIdMap`] をフラットアリーナ形式のバイト列へ直列化する。
    ///
    /// 先頭に [`FORMAT_VERSION`] を埋め込むので、[`from_bytes`](Self::from_bytes) /
    /// [`ArchivedMap::access`] が形式違いを検出できる。
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut nodes: Vec<PersistedNode> = Vec::new();
        let mut dictionary: Vec<Vec<u8>> = Vec::new();
        let mut value_to_idx: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
        let mut empty_idx: Option<u32> = None;

        let lower_root = build_node(
            &self.inner.lower_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );
        let upper_root = build_node(
            &self.inner.upper_root,
            &mut nodes,
            &mut dictionary,
            &mut value_to_idx,
            &mut empty_idx,
        );

        let persisted = PersistedMap {
            version: FORMAT_VERSION,
            lower_root,
            upper_root,
            shard: self.inner.shard.clone(),
            nodes,
            dictionary,
        };
        Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&persisted)
            .map_err(|e| Error::Persist(alloc::format!("serialize: {e}")))?
            .to_vec())
    }

    /// [`to_bytes`](Self::to_bytes) で直列化したバイト列から作業木（`Arc` ベース）を復元する。
    ///
    /// 形式バージョンが [`FORMAT_VERSION`] と異なる場合は
    /// [`Error::UnsupportedFormatVersion`] を返す（誤読させない）。
    ///
    /// # Safety
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let archived = unsafe { rkyv::access_unchecked::<ArchivedPersistedMap>(bytes) };
        check_version(archived.version.to_native())?;
        // 所有型 `PersistedMap` へ復元してから木を組むと、ノード配列と値辞書を
        // まるごと複製することになる。archived 表現を直接読んで組めばその一段が省ける。
        let mut core = FlexTreeCore::<Vec<u8>>::new();
        let empty = core.empty_leaf.clone();
        core.lower_root = rebuild_node(
            archived.lower_root.to_native(),
            &archived.nodes,
            &archived.dictionary,
            &empty,
        );
        core.upper_root = rebuild_node(
            archived.upper_root.to_native(),
            &archived.nodes,
            &archived.dictionary,
            &empty,
        );
        core.shard = archived
            .shard
            .as_ref()
            .map(rkyv::deserialize::<FlexId, rkyv::rancor::Error>)
            .transpose()
            .map_err(|e| Error::Persist(alloc::format!("deserialize shard: {e}")))?;

        Ok(Self { inner: core })
    }
}

/// 作業木の 1 ノードを後行順でアリーナへ書き出し、そのインデックスを返す。
fn build_node(
    node: &SharedNode<Node<Vec<u8>>>,
    nodes: &mut Vec<PersistedNode>,
    dictionary: &mut Vec<Vec<u8>>,
    value_to_idx: &mut BTreeMap<Vec<u8>, u32>,
    empty_idx: &mut Option<u32>,
) -> u32 {
    match &**node {
        Node::Leaf { value: None } => {
            if let Some(i) = *empty_idx {
                i
            } else {
                let i = nodes.len() as u32;
                nodes.push(PersistedNode::Leaf { value: EMPTY_LEAF });
                *empty_idx = Some(i);
                i
            }
        }
        Node::Leaf { value: Some(v) } => {
            let dict_idx = match value_to_idx.get(v) {
                Some(idx) => *idx,
                None => {
                    let idx = dictionary.len() as u32;
                    dictionary.push(v.clone());
                    value_to_idx.insert(v.clone(), idx);
                    idx
                }
            };
            let i = nodes.len() as u32;
            nodes.push(PersistedNode::Leaf {
                value: dict_idx + 1,
            });
            i
        }
        Node::Branch {
            level,
            lower_child,
            upper_child,
            ..
        } => {
            let lower = build_node(lower_child, nodes, dictionary, value_to_idx, empty_idx);
            let upper = build_node(upper_child, nodes, dictionary, value_to_idx, empty_idx);
            let i = nodes.len() as u32;
            nodes.push(PersistedNode::Branch {
                level: *level,
                lower,
                upper,
            });
            i
        }
    }
}

/// archived バイト列を直接走査する ZeroCopy リーダ。`Arc` 木を再構築しない。
pub struct ArchivedMap<'a> {
    inner: &'a ArchivedPersistedMap,
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
    nodes: &'a rkyv::vec::ArchivedVec<ArchivedPersistedNode>,
    node: &'a ArchivedPersistedNode,
}

impl<'a> ArchivedCursor<'a> {
    fn at(nodes: &'a rkyv::vec::ArchivedVec<ArchivedPersistedNode>, idx: u32) -> Self {
        Self {
            nodes,
            node: &nodes[idx as usize],
        }
    }

    /// 値付き葉なら辞書インデックス（1始まり）。分岐・空葉なら `None`。
    fn leaf_value(self) -> Option<u32> {
        match self.node {
            ArchivedPersistedNode::Leaf { value } => match value.to_native() {
                EMPTY_LEAF => None,
                v => Some(v),
            },
            ArchivedPersistedNode::Branch { .. } => None,
        }
    }
}

impl<'a> TreeCursor for ArchivedCursor<'a> {
    fn branch(self) -> Option<(u8, Self, Self)> {
        match self.node {
            ArchivedPersistedNode::Branch {
                level,
                lower,
                upper,
            } => Some((
                *level,
                Self::at(self.nodes, lower.to_native()),
                Self::at(self.nodes, upper.to_native()),
            )),
            ArchivedPersistedNode::Leaf { .. } => None,
        }
    }
}

impl<'a> ArchivedMap<'a> {
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
    /// `bytes` は [`SpatialIdMap::to_bytes`] が生成した正当なバイト列でなければならない。
    pub unsafe fn access(bytes: &'a [u8]) -> Result<Self, Error> {
        let inner = unsafe { rkyv::access_unchecked::<ArchivedPersistedMap>(bytes) };
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

        for (current_id, leaf) in OverlapWalk::new(alloc::vec![root], target.clone()) {
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

/// archived 表現の `idx` を根とする部分木を `Arc` 木へ復元する。
///
/// 所有型 `PersistedMap` を経由しないので、ノード配列と値辞書の複製が丸ごと省ける。
/// 保存していない導出値（`leaf_count` / `max_zoom` / `split_mask`）はここで畳み直す。
fn rebuild_node(
    idx: u32,
    nodes: &rkyv::vec::ArchivedVec<ArchivedPersistedNode>,
    dictionary: &rkyv::vec::ArchivedVec<rkyv::vec::ArchivedVec<u8>>,
    empty: &SharedNode<Node<Vec<u8>>>,
) -> SharedNode<Node<Vec<u8>>> {
    match &nodes[idx as usize] {
        ArchivedPersistedNode::Leaf { value } if value.to_native() == EMPTY_LEAF => empty.clone(),
        ArchivedPersistedNode::Leaf { value } => SharedNode::new(Node::Leaf {
            value: Some(dictionary[(value.to_native() - 1) as usize].to_vec()),
        }),
        ArchivedPersistedNode::Branch {
            level,
            lower,
            upper,
        } => {
            let level = *level;
            let lower_child = rebuild_node(lower.to_native(), nodes, dictionary, empty);
            let upper_child = rebuild_node(upper.to_native(), nodes, dictionary, empty);
            let leaf_count = (lower_child.leaf_count() + upper_child.leaf_count()) as u32;
            let max_zoom = Node::<Vec<u8>>::fold_max_zoom(level, &lower_child, &upper_child);
            let split_mask = Node::<Vec<u8>>::fold_split_mask(level, &lower_child, &upper_child);
            SharedNode::new(Node::Branch {
                level,
                leaf_count,
                max_zoom,
                split_mask,
                lower_child,
                upper_child,
            })
        }
    }
}

#[cfg(test)]
mod version_tests {
    //! 形式バージョンの検証。
    //!
    //! `PersistedMap` の私有フィールドへ触る必要があるので、モジュール内に置く。

    use super::{ArchivedPersistedMap, FORMAT_VERSION, PersistedMap, PersistedNode};
    use crate::{Error, SingleId, SpatialIdMap};
    use alloc::vec::Vec;

    fn sample_bytes() -> Vec<u8> {
        let mut m = SpatialIdMap::new();
        m.insert(SingleId::new(4, 0, 1, 1).unwrap(), alloc::vec![7u8]);
        m.to_bytes().unwrap()
    }

    /// 正しく書かれたバイト列には現行バージョンが入っている。
    #[test]
    fn written_bytes_carry_the_current_version() {
        let bytes = sample_bytes();
        let arch = unsafe { crate::ArchivedMap::access(&bytes) }.unwrap();
        assert_eq!(arch.format_version(), FORMAT_VERSION);
    }

    /// 別バージョンのバイト列を作り、両方の読み出し口が拒否することを確認する。
    ///
    /// これが効いていないと、将来 `PersistedMap` を変更したとき古いバイト列が
    /// 「拒否されず誤って読まれる」ことになる。
    #[test]
    fn foreign_version_is_rejected() {
        let foreign = PersistedMap {
            version: FORMAT_VERSION.wrapping_add(1),
            lower_root: 0,
            upper_root: 0,
            shard: None,
            nodes: alloc::vec![PersistedNode::Leaf { value: 0 }],
            dictionary: Vec::new(),
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&foreign)
            .unwrap()
            .to_vec();

        // 念のため、書いたバージョンが読めていることを確認
        let arch = unsafe { rkyv::access_unchecked::<ArchivedPersistedMap>(&bytes) };
        assert_eq!(arch.version.to_native(), FORMAT_VERSION.wrapping_add(1));

        // ゼロコピー読み出し口
        let Err(err) = (unsafe { crate::ArchivedMap::access(&bytes) }) else {
            panic!("ArchivedMap::access がバージョン違いを通した");
        };
        assert_eq!(
            err,
            Error::UnsupportedFormatVersion {
                expected: FORMAT_VERSION,
                found: FORMAT_VERSION.wrapping_add(1),
            }
        );

        // 全復元の読み出し口
        let err = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes) }.unwrap_err();
        assert!(
            matches!(err, Error::UnsupportedFormatVersion { .. }),
            "from_bytes がバージョン違いを通した: {err:?}"
        );
    }
}
