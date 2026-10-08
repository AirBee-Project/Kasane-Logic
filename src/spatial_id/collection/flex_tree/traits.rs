use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::flex_tree::core::{SafeValue, Summary};
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::source::{Source, SourceIter};
use crate::{Error, FlexId, SpatialIdSet, SpatialIdTable};

impl Source for SpatialIdSet {
    type Value = ();

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        _token: &CancellationToken,
    ) -> Result<SourceIter<'a, ()>, Error> {
        Ok(Box::new(
            self.inner
                .get_overlapping(bounds.iter().copied())
                .map(|(id, ())| Ok((id, ()))),
        ))
    }
}

impl<V, S> Source for SpatialIdTable<V, S>
where
    V: SafeValue + 'static,
    S: Summary<V> + MaybeSendSync + 'static,
{
    type Value = V;

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        _token: &CancellationToken,
    ) -> Result<SourceIter<'a, V>, Error> {
        Ok(Box::new(
            self.inner
                .get_overlapping(bounds.iter().copied())
                .map(|(id, value)| Ok((id, value.clone()))),
        ))
    }
}

// ---------------------------------------------------------------------------
// クエリ結果を具象コレクションで受け取るための入口
//
// 実行メソッドは「検証・最適化するか」×「何で受け取るか」の2軸でできている。
//
// |                                | 検証・最適化あり | AST の順序のまま |
// |--------------------------------|-----------------|------------------|
// | `SpatialIdTable<V, S>`         | `run`           | `raw_run`        |
// | `SpatialIdSet`                 | `run_set`       | `raw_run_set`    |
// | `SpatialIdTable<V, NoSummary>` | `run_table`     | `raw_run_table`  |
//
// `raw_*` は「AST を組み替えず、書かれた順序のまま実行する」を意味する。テストや
// ベンチで最適化の有無を比べるための口であり、通常は左列を使う。
//
// クエリは内部で `SpatialIdTable<V, NoSummary>` として実行する。`run` は結果を
// 指定の Summary で組み直すので木の再構築がかかる。`run_set` と `run_table` は
// 包み直すだけでコストはかからない。
// ---------------------------------------------------------------------------

impl<V: SafeValue + 'static> Query<V> {
    /// 検証・最適化して実行し、Summary `S` の [`SpatialIdTable`] として返す。
    ///
    /// 結果を走査するだけなら、組み直しの要らない [`run_table`](Query::run_table) のほうが速い。
    pub fn run<S: Summary<V>>(self) -> Result<SpatialIdTable<V, S>, Error> {
        Ok(self.run_table()?.into_iter().collect())
    }

    /// 検証も最適化もせず実行し、Summary `S` の [`SpatialIdTable`] として返す。
    pub fn raw_run<S: Summary<V>>(self) -> Result<SpatialIdTable<V, S>, Error> {
        Ok(self.raw_run_table()?.into_iter().collect())
    }
}

impl Query<()> {
    /// 検証・最適化して実行し、[`SpatialIdSet`] として返す。集合へは木を包み直すだけでコストはかからない。
    pub fn run_set(self) -> Result<SpatialIdSet, Error> {
        Ok(SpatialIdSet {
            inner: self.run_table()?.inner,
            shard: None,
        })
    }

    /// 検証も最適化もせず実行し、[`SpatialIdSet`] として返す。
    pub fn raw_run_set(self) -> Result<SpatialIdSet, Error> {
        Ok(SpatialIdSet {
            inner: self.raw_run_table()?.inner,
            shard: None,
        })
    }
}
