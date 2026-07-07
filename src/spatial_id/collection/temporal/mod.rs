//! 時空間コア層（[`SpatioTemporalCore`]）。
//!
//! [`FlexTree`](crate::FlexTree)（純空間の木）と公開コレクション
//! （[`SpatialIdSet`](crate::SpatialIdSet) / [`SpatialIdMap`](crate::SpatialIdMap) /
//! [`SpatialIdTable`](crate::SpatialIdTable)）の間に挟まる中間層で、
//! 時間軸（[`TemporalSet`] / [`TemporalMap`]）に関するすべての操作をここに集約し、
//! `FlexTree` を純粋な空間インデックスとして保つ。
//!
//! - `combine`: 時間値の合成規則（[`Combine`] 実装群）
//! - `value`: 葉の値の抽象（[`TemporalValue`]）
//! - 本モジュール: [`SpatioTemporalCore`]（挿入・クエリ・削除・列挙の共通ロジック）

pub(crate) mod combine;

pub(crate) use combine::{TMapDifference, TMapIntersection, TMapOverwrite};

use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::node_ops::Combine;
use crate::{FlexId, FlexTree, SpatialId, TemporalSet};

// ─── SpatioTemporalCore ───────────────────────────────────────────────────────

/// 時空間コア層。
///
/// [`FlexTree<TV>`] を内包し、時間軸（挿入・クエリ・結合）のロジックを集約する。
/// `TV` は [`TemporalSet`] または [`TemporalMap<V>`](TemporalMap) のいずれか。
///
/// [`SpatialIdSet`](crate::SpatialIdSet), [`SpatialIdMap`](crate::SpatialIdMap),
/// [`SpatialIdTable`](crate::SpatialIdTable) はこの型を内包して使う。
#[derive(Clone, Debug, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "persist", rkyv(archive_bounds(V: 'static)))]
#[cfg_attr(
    feature = "persist",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::ser::Sharing,
        <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(deserialize_bounds(
        __D: rkyv::de::Pooling,
        <__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext + rkyv::validation::SharedContext,
        <__C as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    )))
)]
pub(crate) struct SpatioTemporalCore<
    V: Clone + PartialEq + crate::spatial_id::collection::flex_tree::ptr::SafeValue,
> {
    pub(crate) inner: FlexTree<crate::TemporalMap<V>>,
}

impl<V: Clone + PartialEq + crate::spatial_id::collection::flex_tree::ptr::SafeValue>
    SpatioTemporalCore<V>
{
    /// 空の [`SpatioTemporalCore`] を作成する。
    pub(crate) fn new() -> Self {
        Self {
            inner: FlexTree::new(),
        }
    }

    /// シャード領域に閉じた空の [`SpatioTemporalCore`] を作成する。
    pub(crate) fn new_in_shard(region: FlexId) -> Self {
        Self {
            inner: FlexTree::new_in_shard(region),
        }
    }

    /// シャード領域を返す。
    pub(crate) fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }

    /// シャードの分割が必要か判定する。
    pub(crate) fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    /// シャードを分割する。
    pub(crate) fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let ((lower_region, lower_inner), (upper_region, upper_inner)) =
            self.inner.split_shard()?;
        Some((
            (lower_region, Self { inner: lower_inner }),
            (upper_region, Self { inner: upper_inner }),
        ))
    }

    /// `combine_with` の委譲。
    pub(crate) fn combine_with<C: Combine<crate::TemporalMap<V>>>(
        &self,
        other: &Self,
        shard: Option<FlexId>,
    ) -> Self {
        Self {
            inner: self.inner.combine_with::<C>(&other.inner, shard),
        }
    }

    /// 空かどうか。
    pub(crate) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 時間セルの個数を返す（iter() が返すセル数と一致）。
    ///
    /// 各葉の [`count_cells`](crate::TemporalMap::count_cells) を合算するだけで、
    /// セルを1つも生成しない（O(空間ノード数 × セグメント数)、割当なし）。
    pub(crate) fn count(&self) -> usize {
        self.inner.iter_ref().map(|(_, tv)| tv.count_cells()).sum()
    }

    /// ツリーの最大ズームレベルを返す。
    pub(crate) fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 内部をすべてクリアする。
    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }

    /// テスト用：ルートノードのポインタが同一か確認する。
    #[cfg(test)]
    pub(crate) fn root_ptr_eq(&self, other: &Self) -> bool {
        self.inner.root_ptr_eq(&other.inner)
    }

    // ── 挿入 ──────────────────────────────────────────────────────────────────

    /// シャードを考慮して `flex_id` を挿入する汎用ロジック。
    ///
    /// - 全時間（WHOLE）の場合は `insert_flex_id` で直接置換する。
    /// - それ以外は `InsertCombine` で既存の時間と合成する。
    pub(crate) fn insert_flex_id(&mut self, flex_id: FlexId, payload: V) {
        // シャード領域外は無視し、はみ出しは切り詰める。
        let flex_id = match self.inner.shard() {
            Some(region) => match flex_id.intersection(region) {
                Some(clipped) => clipped,
                None => return,
            },
            None => flex_id,
        };
        let temporal = flex_id.temporal().clone();
        let spatial = flex_id.spatial_part();

        if temporal.is_whole() {
            // 全時間は覆う領域を直接置換するだけで InsertCombine と同じ結果になる。
            let mut tm = crate::TemporalMap::new();
            tm.insert(&crate::TemporalId::WHOLE, payload);
            self.inner.insert_flex_id(spatial, tm);
        } else {
            // 有限時間はインプレースで既存の時間マップへ後勝ちマージする。
            // 使い捨て of 単一要素木 + combine_with を経由しないため定数倍が軽い。
            let mut tv = crate::TemporalMap::new();
            tv.insert(&temporal, payload);
            self.inner
                .insert_combine_mut::<TMapOverwrite>(&spatial, &tv);
        }
    }

    // ── クエリ ────────────────────────────────────────────────────────────────

    /// `target` と空間・時間の両方で交差する `(FlexId, &Payload)` を返す。
    ///
    /// 空間は `target` の範囲に切り取られ、時間は `target` の時間との交差が付く。
    pub(crate) fn get<'a, S: SpatialId>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.get_ref(target).flat_map(|(clipped, tv)| {
            tv.cells_clipped_ref_iter(clipped.temporal().clone())
                .map(move |(t, p)| (clipped.clone().with_temporal(t), p))
        })
    }

    /// `target` と空間・時間の両方で交差する葉を参照として返す。
    pub(crate) fn get_overlapping<'a, S: SpatialId + 'a>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let query_temporal = target.temporal().clone();
        self.inner
            .get_overlapping_ref(target)
            .flat_map(move |(stored, tv)| {
                tv.cells_clipped_ref_iter(query_temporal.clone())
                    .map(move |(t, p)| (stored.clone().with_temporal(t), p))
            })
    }

    /// `target` と空間・時間の両方で隣接（面共有）する葉を参照として返す。
    pub(crate) fn neighbors_share_face<'a, S: SpatialId + 'a>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let query_temporal = target.temporal().clone();
        self.inner
            .neighbors_share_face_ref(target)
            .flat_map(move |(stored, tv)| {
                tv.cells_clipped_ref_iter(query_temporal.clone())
                    .map(move |(t, p)| (stored.clone().with_temporal(t), p))
            })
    }

    // ── 列挙 ──────────────────────────────────────────────────────────────────

    /// 保持するすべての `(FlexId, &Payload)` を返す。
    pub(crate) fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter_ref().flat_map(|(spatial, tv)| {
            tv.cells_ref_iter()
                .map(move |(t, p)| (spatial.clone().with_temporal(t), p))
        })
    }

    // ── 削除 ──────────────────────────────────────────────────────────────────

    /// `target` と空間・時間の両方で交差する部分を切り取り削除して返す。
    ///
    /// 空間的に重なる葉を取り出し、query の空間外の残余と query の時間外の残余を
    /// 木へ戻す（外科的削除）。
    pub(crate) fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed = Vec::new();
        for query in target.iter_flex_ids() {
            let q_spatial = query.spatial_part();
            let q_time = TemporalSet::from(query.temporal());
            let affected: Vec<(FlexId, crate::TemporalMap<V>)> =
                self.inner.remove_overlapping(&q_spatial).collect();
            for (leaf, tv) in affected {
                // query の空間外の残余はそのまま戻す。
                for remnant in leaf.difference(&q_spatial) {
                    self.inner.insert_flex_id(remnant, tv.clone());
                }
                // 空間交差部は時間で分割する。
                if let Some(inter) = leaf.intersection(&q_spatial) {
                    let kept = tv.subtract_time(&q_time);
                    if !kept.is_empty() {
                        self.inner.insert_flex_id(inter.clone(), kept);
                    }
                    for (t, p) in tv.intersect_time(&q_time).cells() {
                        removed.push((inter.with_temporal(t), p));
                    }
                }
            }
        }
        removed.into_iter()
    }

    /// `target` と空間・時間の両方で交差する葉を削除して返す。
    ///
    /// `remove` とは異なり、空間的な残余の切り出しは行わず、交差した葉自体を取り除きます。
    /// ただし、時間の残余（ターゲット時間外の部分）は同じ空間葉として木に戻されます。
    pub(crate) fn remove_overlapping<S: SpatialId>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed = Vec::new();
        for query in target.iter_flex_ids() {
            let q_spatial = query.spatial_part();
            let q_time = TemporalSet::from(query.temporal());
            let affected: Vec<(FlexId, crate::TemporalMap<V>)> =
                self.inner.remove_overlapping(&q_spatial).collect();
            for (leaf, tv) in affected {
                let kept = tv.subtract_time(&q_time);
                if !kept.is_empty() {
                    self.inner.insert_flex_id(leaf.clone(), kept);
                }
                for (t, p) in tv.intersect_time(&q_time).cells() {
                    removed.push((leaf.clone().with_temporal(t), p));
                }
            }
        }
        removed.into_iter()
    }

    // ── シャード合成規則 ──────────────────────────────────────────────────────

    /// union のシャード合成規則。
    pub(crate) fn shard_for_union(a: &Self, b: &Self) -> Option<FlexId> {
        FlexTree::shard_for_union(&a.inner, &b.inner)
    }

    /// intersection のシャード合成規則。
    pub(crate) fn shard_for_intersection(a: &Self, b: &Self) -> Option<FlexId> {
        FlexTree::shard_for_intersection(&a.inner, &b.inner)
    }
}
