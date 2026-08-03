use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::source::Source;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, FlexId, RangeId, SpatialIdSet, SpatialIdTable};

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

    fn read_subset(&self, bounds: &[RangeId]) -> Result<WorkingTree<()>, Error> {
        let mut time_segments: Vec<(FlexId, ())> = Vec::new();
        for b in bounds {
            for id in self.get_range(b) {
                time_segments.push((id, ()));
            }
        }
        Ok(time_segments.into_iter().collect())
    }

    fn read_all(self: Box<Self>) -> Result<WorkingTree<()>, Error> {
        // 所有権ごと移し替えるだけ（クローンしない）。
        Ok(WorkingTree::from_core(SpatialIdSet::into_core(*self)))
    }
}

impl From<WorkingTree<()>> for SpatialIdSet {
    /// 包み直すだけでコストはかからない。
    fn from(working: WorkingTree<()>) -> Self {
        SpatialIdSet::from_core(working.into_core())
    }
}

impl<V> Source for SpatialIdTable<V>
where
    V: FlexIdValue + 'static,
{
    type Value = V;

    fn read_subset(&self, bounds: &[RangeId]) -> Result<WorkingTree<V>, Error> {
        let mut time_segments: Vec<(FlexId, V)> = Vec::new();
        for b in bounds {
            for (id, value) in self.get_range(b) {
                time_segments.push((id, value.clone()));
            }
        }
        Ok(time_segments.into_iter().collect())
    }

    fn read_all(self: Box<Self>) -> Result<WorkingTree<V>, Error> {
        // rank ツリーを辞書で実体値へ展開する。ランク → 実体値は単射なので、木の形は
        // まったく変わらない。平坦化して組み直す必要はなく、値だけを写せばよい。
        //
        // 引きは葉ごとに走る。`BTreeMap` を葉の数だけ降りるとポインタ追跡が効くので、
        // 木へ入る前にランク添字の密な表へ均しておく。
        let by_rank = self.values_by_rank();
        Ok(WorkingTree::from_core(
            self.rank_core().map_values_injective(&|rank: &usize| {
                by_rank[*rank]
                    .expect("ツリー内のランクは必ず逆引き辞書にある")
                    .clone()
            }),
        ))
    }
}

impl<V> From<WorkingTree<V>> for SpatialIdTable<V>
where
    V: FlexIdValue + 'static,
{
    /// 実体値のSegmentを辞書へ intern し直す。
    ///
    /// 実体値 → ランクは単射なので、[`read_all`](Source::read_all) と同じく木の形は
    /// 変わらない。出現値を集めて辞書を作り、木は値だけを写す。
    fn from(working: WorkingTree<V>) -> Self {
        let core = working.into_core();
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

// ---------------------------------------------------------------------------
// クエリ結果を具象コレクションで受け取るための便宜メソッド
//
// [`Query::run`] / [`Query::raw_run`] が返すのは作業木（[`FlexTreeCore`]）そのもの。
// 具象コレクションへの変換にはコスト（辞書への再 intern とツリー再構築）がかかるため、
// 変換するかどうかは呼び出し側が選べるようにしてある。以下はその変換込みの入口で、
// `q.run()?.into()` と等価。
// ---------------------------------------------------------------------------

impl<V: SafeValue + Ord + 'static> Query<V> {
    /// 検証・最適化して実行し、[`SpatialIdTable`] として返す。
    ///
    /// `q.run()?.into()` と等価。結果を走査するだけなら [`run`](Query::run) の
    /// 戻り値をそのまま使うほうが、辞書への再 intern の分だけ速い。
    pub fn run_table(self) -> Result<SpatialIdTable<V>, Error> {
        Ok(self.run()?.into())
    }

    /// 検証も最適化もせず実行し、[`SpatialIdTable`] として返す。
    ///
    /// `q.raw_run()?.into()` と等価。
    pub fn raw_run_table(self) -> Result<SpatialIdTable<V>, Error> {
        Ok(self.raw_run()?.into())
    }
}

impl Query<()> {
    /// 検証・最適化して実行し、[`SpatialIdSet`] として返す。
    ///
    /// `q.run()?.into()` と等価。集合への変換は包み直すだけでコストはかからない。
    pub fn run_set(self) -> Result<SpatialIdSet, Error> {
        Ok(self.run()?.into())
    }

    /// 検証も最適化もせず実行し、[`SpatialIdSet`] として返す。
    ///
    /// `q.raw_run()?.into()` と等価。
    pub fn raw_run_set(self) -> Result<SpatialIdSet, Error> {
        Ok(self.raw_run()?.into())
    }
}
