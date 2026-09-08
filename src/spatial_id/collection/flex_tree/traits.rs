use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::{FlexTreeCore, SafeValue};
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{Query, Source, ValueIter};
use crate::{Error, RangeId, SpatialIdSet, SpatialIdTable};

/// Table の出入口変換で、これ未満なら rayon を使わず逐次で組む閾値。
/// 単発・小規模クエリで rayon 起動コスト（par_build / from_par_iter の par_sort 等）を避ける。
#[cfg(feature = "rayon")]
const SEQ_CONVERT_THRESHOLD: usize = 512;

#[cfg(not(feature = "rayon"))]
pub trait FlexIdValue: Ord + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: Ord + Clone> FlexIdValue for T {}

#[cfg(feature = "rayon")]
pub trait FlexIdValue: Ord + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: Ord + Clone + Send + Sync> FlexIdValue for T {}

impl Source for SpatialIdSet {
    type Value = ();

    fn get<'a>(
        &'a self,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, ()>, Error> {
        let mut counter = 0u32;
        Ok(Box::new(self.get_range(&target).map_while(move |id| {
            token.check_amortized(&mut counter).ok().map(|_| (id, ()))
        })))
    }
}

impl From<FlexTreeCore<()>> for SpatialIdSet {
    /// 包み直すだけでコストはかからない。
    fn from(core: FlexTreeCore<()>) -> Self {
        SpatialIdSet::from_core(core)
    }
}

impl<V> Source for SpatialIdTable<V>
where
    V: FlexIdValue + 'static,
{
    type Value = V;

    fn get<'a>(
        &'a self,
        target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let mut counter = 0u32;
        Ok(Box::new(self.get_range(&target).map_while(
            move |(id, v)| {
                token
                    .check_amortized(&mut counter)
                    .ok()
                    .map(|_| (id, v.clone()))
            },
        )))
    }
}

impl<V> From<FlexTreeCore<V>> for SpatialIdTable<V>
where
    V: FlexIdValue + 'static,
{
    /// 実体値のSegmentを辞書へ intern し直す。
    ///
    /// 実体値 → ランクは単射なので、木の形は変わらない。出現値を集めて辞書を作り、
    /// 木は値だけを写す。
    fn from(core: FlexTreeCore<V>) -> Self {
        if core.is_empty() {
            return SpatialIdTable::new();
        }

        // 1. 出現値を集めてソート＋重複排除し、決定的なランク順（1 始まり）を得る。
        let mut values: Vec<V> = core.iter_ref().map(|(_, v)| v.clone()).collect();
        #[cfg(feature = "rayon")]
        {
            use rayon::prelude::*;
            if values.len() >= SEQ_CONVERT_THRESHOLD {
                values.par_sort_unstable();
            } else {
                values.sort_unstable();
            }
        }
        #[cfg(not(feature = "rayon"))]
        values.sort_unstable();
        values.dedup();

        // 2. 木は形を保ったままランクへ写す。
        let ranks = core.map_values_injective(&|v: &V| values.binary_search(v).unwrap() + 1);

        SpatialIdTable::from_ranked_core(ranks, values)
    }
}

impl<V: SafeValue + Ord + 'static> Query<V> {
    /// クエリを実行し、結果を [`SpatialIdTable`] へ集約する。
    pub fn collect_table(&self) -> Result<SpatialIdTable<V>, Error> {
        let tree: FlexTreeCore<V> = self.run()?.collect();
        Ok(tree.into())
    }

    /// 対象領域内のクエリを実行し、結果を [`SpatialIdTable`] へ集約する。
    pub fn collect_table_within(
        &self,
        target: impl Into<RangeId>,
    ) -> Result<SpatialIdTable<V>, Error> {
        let tree: FlexTreeCore<V> = self.run_within(target)?.collect();
        Ok(tree.into())
    }
}

impl Query<()> {
    /// クエリを実行し、結果を [`SpatialIdSet`] へ集約する。
    ///
    /// [`collect_table`](Query::collect_table) と同名にできないのは、`()` も `Ord` を
    /// 満たすため `impl<V: Ord> Query<V>` の実装と重なって inherent impl の衝突（E0592）に
    /// なるため。
    pub fn collect_set(&self) -> Result<SpatialIdSet, Error> {
        let tree: FlexTreeCore<()> = self.run()?.collect();
        Ok(tree.into())
    }

    /// 対象領域内のクエリを実行し、結果を [`SpatialIdSet`] へ集約する。
    pub fn collect_set_within(&self, target: impl Into<RangeId>) -> Result<SpatialIdSet, Error> {
        let tree: FlexTreeCore<()> = self.run_within(target)?.collect();
        Ok(tree.into())
    }
}

