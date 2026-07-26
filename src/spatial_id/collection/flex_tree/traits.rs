use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::source::Source;
use crate::{Error, FlexId, FlexTreeCore, RangeId, SpatialIdSet, SpatialIdTable};

/// Table の出入口変換で、これ未満なら rayon を使わず逐次で組む閾値。
/// 単発・小規模クエリで rayon 起動コスト（par_build / from_par_iter の par_sort 等）を避ける。
#[cfg(feature = "rayon")]
const SEQ_CONVERT_THRESHOLD: usize = 512;

#[cfg(not(feature = "rayon"))]
pub trait CellValue: Ord + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: Ord + Clone> CellValue for T {}

#[cfg(feature = "rayon")]
pub trait CellValue: Ord + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
impl<T: Ord + Clone + Send + Sync> CellValue for T {}

impl Source for SpatialIdSet {
    type Value = ();

    fn read_subset(&self, bounds: &[RangeId]) -> Result<FlexTreeCore<()>, Error> {
        let mut cells: Vec<(FlexId, ())> = Vec::new();
        for b in bounds {
            for id in self.get_range(b) {
                cells.push((id, ()));
            }
        }
        Ok(cells.into_iter().collect())
    }

    fn read_all(self: Box<Self>) -> Result<FlexTreeCore<()>, Error> {
        // 所有権ごと移し替えるだけ（クローンしない）。
        Ok(SpatialIdSet::into_core(*self))
    }
}

impl From<FlexTreeCore<()>> for SpatialIdSet {
    fn from(working: FlexTreeCore<()>) -> Self {
        SpatialIdSet::from_core(working)
    }
}

impl<V> Source for SpatialIdTable<V>
where
    V: CellValue + 'static,
{
    type Value = V;

    fn read_subset(&self, bounds: &[RangeId]) -> Result<FlexTreeCore<V>, Error> {
        let mut cells: Vec<(FlexId, V)> = Vec::new();
        for b in bounds {
            for (id, value) in self.get_range(b) {
                cells.push((id, value.clone()));
            }
        }
        Ok(cells.into_iter().collect())
    }

    fn read_all(self: Box<Self>) -> Result<FlexTreeCore<V>, Error> {
        // rank ツリーを辞書で実体値へ展開。Table のセルは互いに素なので union（par_build_vec）で正しい。
        #[cfg(feature = "rayon")]
        {
            let items: Vec<(FlexId, V)> = (*self).into_iter().collect();
            // 小入力では rayon 起動コストが利得を上回るので逐次挿入で組む。
            if items.len() < SEQ_CONVERT_THRESHOLD {
                let mut core = FlexTreeCore::new();
                for (id, value) in items {
                    core.insert(id, value);
                }
                Ok(core)
            } else {
                Ok(FlexTreeCore::par_build_vec(items))
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            let mut core = FlexTreeCore::new();
            for (id, value) in *self {
                core.insert(id, value);
            }
            Ok(core)
        }
    }
}

impl<V> From<FlexTreeCore<V>> for SpatialIdTable<V>
where
    V: CellValue + 'static,
{
    /// 実体値の互いに素なセルを辞書へ intern し直す。小入力は逐次で（rayon 起動コスト回避）。
    fn from(core: FlexTreeCore<V>) -> Self {
        #[cfg(feature = "rayon")]
        {
            let cells: Vec<(FlexId, V)> = core.into_iter().collect();
            use rayon::iter::FromParallelIterator;
            if cells.len() < SEQ_CONVERT_THRESHOLD {
                cells.into_iter().collect()
            } else {
                SpatialIdTable::from_par_iter(cells)
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            core.into_iter().collect()
        }
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
