use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::traits::{BinaryOperator, QueryIter};
use crate::{Error, SpatialIdTable};
use alloc::boxed::Box;

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
        let mut target_a: SpatialIdTable<V, NoSummary> = lhs.collect::<Result<_, _>>()?;
        let target_b: SpatialIdTable<V, NoSummary> = rhs.collect::<Result<_, _>>()?;
        target_a.inner = target_a.inner.intersection(&target_b.inner);
        Ok(Box::new(target_a.into_iter().map(Ok)))
    }

    fn reverse_target(
        &self,
        output_target: crate::RangeId,
    ) -> (Option<crate::RangeId>, Option<crate::RangeId>) {
        (Some(output_target.clone()), Some(output_target))
    }
}
