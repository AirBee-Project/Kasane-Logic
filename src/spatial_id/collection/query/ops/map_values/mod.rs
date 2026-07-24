#[cfg(test)]
mod test;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::{execution::Query, source::Source, traits::WorkingTree};
use crate::{Error, FlexId, FlexTreeCore, RangeId};

/// 部分クエリ `inner` の結果の値を `f` で写し、別の値型の作業木として読み出す入力源。
pub struct MapValues<W: WorkingTree + 'static, U, F> {
    inner: Query<W>,
    f: F,
    _marker: core::marker::PhantomData<fn() -> U>,
}

impl<W, U, F> MapValues<W, U, F>
where
    W: WorkingTree + 'static,
    U: SafeValue,
    F: Fn(W::Value) -> U,
{
    pub fn new(inner: Query<W>, f: F) -> Self {
        Self {
            inner,
            f,
            _marker: core::marker::PhantomData,
        }
    }

    /// 変換後のセル列を組み立てる。
    fn map_cells(&self, working: W) -> FlexTreeCore<U> {
        let cells: Vec<(FlexId, U)> = working
            .into_iter()
            .map(|(id, value)| (id, (self.f)(value)))
            .collect();
        cells.into_iter().collect()
    }
}

impl<W, U, F> Source for MapValues<W, U, F>
where
    W: WorkingTree + 'static,
    U: SafeValue + 'static,
    F: Fn(W::Value) -> U + MaybeSendSync + 'static,
    W::Value: 'static,
{
    type Working = FlexTreeCore<U>;

    fn read_subset(&self, bounds: &[RangeId]) -> Result<Self::Working, Error> {
        let working = self.inner.run_on_subset(bounds.to_vec())?;
        Ok(self.map_cells(working))
    }

    fn read_all(self: Box<Self>) -> Result<Self::Working, Error> {
        let this = *self;
        let working = this.inner.raw_run_working_tree()?;
        // `map_cells` は &self を取るので、写像関数だけを取り出して使う。
        let cells: Vec<(FlexId, U)> = working
            .into_iter()
            .map(|(id, value)| (id, (this.f)(value)))
            .collect();
        Ok(cells.into_iter().collect())
    }
}

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: 'static,
{
    /// 各空間の値を `f` で写し、**別の値**へ変換する。
    ///
    /// ```ignore
    /// // 数値テーブルを「閾値超えなら true」の真偽値クエリへ写す
    /// let q = table.query().map_values(|v: u32| v > 10);
    /// ```
    pub fn map_values<U, F>(self, f: F) -> Query<FlexTreeCore<U>>
    where
        U: SafeValue + 'static,
        F: Fn(W::Value) -> U + MaybeSendSync + 'static,
    {
        Query::Source(Box::new(MapValues::new(self, f)))
    }
}
