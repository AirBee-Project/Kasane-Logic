use crate::Error;
use crate::spatial_id::collection::query::ops::binary::collect_both;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::traits::{BinaryOperator, QueryIter};

pub struct Intersection<V> {
    _marker: core::marker::PhantomData<V>,
}

impl<V> Intersection<V> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V> Default for Intersection<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: SafeValue + 'static> BinaryOperator<V> for Intersection<V> {
    fn run<'a>(
        &self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        let (mut target_a, target_b) = collect_both(lhs, rhs)?;
        target_a.inner = target_a.inner.intersection(&target_b.inner);
        Ok(target_a.into())
    }
}
