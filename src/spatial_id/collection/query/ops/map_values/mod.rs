//! 値の型を変える写像。
//!
//! `Query<W>` は単一の作業型 `W` で型付けされるため、値型を変える演算子を AST の途中に
//! 単項演算子として置くことはできない（`UnaryOperator<W>` は入出力が同型）。
//!
//! 一方、「範囲を渡すと変換後の作業木を返すもの」は [`Source`] の契約そのものなので、
//! **型変換付きの部分クエリを1つの入力源として包む**ことで実現する。この方法なら
//! `Query` の構造・最適化器・検証には一切手を入れずに済む。

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
        // 値の写像は空間的な形を変えないので、要求領域はそのまま内側へ渡してよい。
        let working = self.inner.run_on_subset(bounds.to_vec())?;
        Ok(self.map_cells(working))
    }

    fn read_all(self: Box<Self>) -> Result<Self::Working, Error> {
        let this = *self;
        let working = this.inner.raw_run()?;
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
    /// 各セルの値を `f` で写し、**別の値型のクエリ**へ変換する。
    ///
    /// 型が変わる地点は最適化の境界になる（`f` の意味を実行器が解釈できないため、
    /// これより内側と外側で演算子の並べ替えは行われない）。
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
