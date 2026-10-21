#[cfg(test)]
mod test;

use alloc::boxed::Box;

use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::query_iter::QueryIter;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{Query, traits::Source};
use crate::{Error, RangeId};

/// `inner` の各値を `f` で変換する。`UnaryOperator<V>` は値の型を変えられないので、`Source` として実装する。
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
        targets: &[RangeId],
        token: &CancellationToken,
    ) -> Result<QueryIter<'a, U>, Error> {
        // `Query::validate` は入力源の中まで辿らないので、内側のクエリはここで検証する
        self.inner.validate()?;
        let inner = self.inner.stream_within(targets.to_vec(), token)?;
        Ok(inner.filter_map_ordered(move |value| Some((self.f)(value))))
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
