#[cfg(test)]
mod test;

use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::source::SourceIter;
use crate::spatial_id::collection::query::{execution::Query, source::Source};
use crate::{Error, FlexId, RangeId};

pub struct MapValues<V: SafeValue + 'static, U, F> {
    inner: Query<V>,
    f: F,
    _marker: core::marker::PhantomData<fn() -> U>,
}

impl<V, U, F> MapValues<V, U, F>
where
    V: SafeValue + 'static,
    U: SafeValue,
    F: Fn(V) -> U,
{
    pub fn new(inner: Query<V>, f: F) -> Self {
        Self {
            inner,
            f,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V, U, F> Source for MapValues<V, U, F>
where
    V: SafeValue + 'static,
    U: SafeValue + 'static,
    F: Fn(V) -> U + MaybeSendSync + 'static,
{
    type Value = U;

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        token: &CancellationToken,
    ) -> Result<SourceIter<'a, U>, Error> {
        Ok(Box::new(
            self.inner
                .run_within(bounds.iter().map(RangeId::from).collect(), token)?
                .into_iter()
                .map(move |(id, value)| Ok((id, (self.f)(value)))),
        ))
    }
}

impl<V: SafeValue + 'static> Query<V> {
    /// 各空間の値を `f` で写し、**別の値型**のクエリへ変換する。
    ///
    /// ```ignore
    /// // 数値テーブルを「閾値超えなら true」の真偽値クエリへ写す
    /// let q = table.query().map_values(|v: u32| v > 10);
    /// ```
    pub fn map_values<U, F>(self, f: F) -> Query<U>
    where
        U: SafeValue + 'static,
        F: Fn(V) -> U + MaybeSendSync + 'static,
    {
        Query::Source(Box::new(MapValues::new(self, f)))
    }
}
