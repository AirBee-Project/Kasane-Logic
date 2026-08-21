use crate::spatial_id::collection::query::working::WorkingTree;
#[cfg(test)]
mod test;

use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{execution::Query, source::Source};
use crate::{Error, RangeId};

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

    fn read_range_ids(
        &self,
        bounds: &[RangeId],
        token: &CancellationToken,
    ) -> Result<WorkingTree<U>, Error> {
        Ok(self
            .inner
            .run_within(bounds.to_vec(), token)?
            .into_iter()
            .map(|(id, value)| (id, (self.f)(value)))
            .collect())
    }

    fn read_range_ids_flat(
        &self,
        bounds: &[RangeId],
        token: &CancellationToken,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, U)>, Error> {
        // Query::run_within_flat を呼ぶのが理想ですが、今は run_within を使って Vec にします。
        // 後で Query 側も完全にフラット化します。
        Ok(self
            .inner
            .run_within(bounds.to_vec(), token)?
            .into_iter()
            .map(|(id, value)| (id, (self.f)(value)))
            .collect())
    }

    fn read_all(self: Box<Self>, token: &CancellationToken) -> Result<WorkingTree<U>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let this = *self;
        Ok(this
            .inner
            .run_working_tree()?
            .into_iter()
            .map(|(id, value)| (id, (this.f)(value)))
            .collect())
    }

    fn read_all_flat(
        self: Box<Self>,
        token: &CancellationToken,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, U)>, Error> {
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let this = *self;
        Ok(this
            .inner
            .raw_run_working_tree()?
            .into_iter()
            .map(|(id, value)| (id, (this.f)(value)))
            .collect())
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
