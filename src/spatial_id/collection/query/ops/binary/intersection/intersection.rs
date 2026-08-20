use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{
    Error,
    spatial_id::collection::{flex_tree::core::SafeValue, query::traits::BinaryOperator},
};

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
    fn run(&self, target_a: &mut WorkingTree<V>, target_b: &WorkingTree<V>) -> Result<(), Error> {
        if target_a.core().count() == 0 {
            return Ok(());
        }
        if target_b.core().count() == 0 {
            target_a.core_mut().clear();
            return Ok(());
        }

        // A ∩ B = A - (A - B)
        // これにより、Aの要素のValueを完全に維持しながら、AとBが重複する領域だけを残すことができます。
        let not_b = target_a.core().difference(target_b.core());
        let intersection = target_a.core().difference(&not_b);
        *target_a = WorkingTree::from_core(intersection);
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
