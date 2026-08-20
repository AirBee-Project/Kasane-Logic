use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{
    Error,
    spatial_id::collection::{flex_tree::core::SafeValue, query::traits::BinaryOperator},
};

pub struct Difference<V> {
    _marker: core::marker::PhantomData<V>,
}

impl<V> Difference<V> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V> Default for Difference<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: SafeValue> BinaryOperator<V> for Difference<V> {
    fn run(&self, target_a: &mut WorkingTree<V>, target_b: &WorkingTree<V>) -> Result<(), Error> {
        if target_a.core().count() == 0 {
            return Ok(());
        }
        if target_b.core().count() == 0 {
            return Ok(());
        }
        let diff = target_a.core().difference(target_b.core());
        *target_a = WorkingTree::from_core(diff);
        Ok(())
    }

    fn inverse_bounds(
        &self,
        output_bounds: crate::RangeId,
    ) -> (Option<crate::RangeId>, Option<crate::RangeId>) {
        (Some(output_bounds.clone()), Some(output_bounds))
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "difference")
    }
}
