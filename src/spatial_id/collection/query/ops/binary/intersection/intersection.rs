use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::traits::BinaryOperator;
use crate::{Error, SpatialIdTable};

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

impl<V: SafeValue> BinaryOperator<V> for Intersection<V> {
    fn run(
        &self,
        target_a: &mut SpatialIdTable<V, NoSummary>,
        target_b: &SpatialIdTable<V, NoSummary>,
    ) -> Result<(), Error> {
        target_a.inner = target_a.inner.intersection(&target_b.inner);
        Ok(())
    }

    fn inverse_bounds(
        &self,
        output_bounds: crate::RangeId,
    ) -> (Option<crate::RangeId>, Option<crate::RangeId>) {
        (Some(output_bounds.clone()), Some(output_bounds))
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "intersection")
    }
}
