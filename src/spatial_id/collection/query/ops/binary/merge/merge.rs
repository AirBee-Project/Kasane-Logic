use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{
    Error,
    spatial_id::collection::{
        flex_tree::core::SafeValue,
        query::{merge_policy::MergePolicy, traits::BinaryOperator},
    },
};

/// 2つの作業木を `MergePolicy` で重ね合わせる二項演算子。
pub struct Merge<V, P> {
    default: V,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<V, P> Merge<V, P> {
    pub fn new(default: V) -> Self {
        Self {
            default,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V: SafeValue, P> BinaryOperator<V> for Merge<V, P>
where
    P: MergePolicy<V>,
{
    fn run(&self, target_a: &mut WorkingTree<V>, target_b: &WorkingTree<V>) -> Result<(), Error> {
        if target_a.core().count() == 0 && target_b.core().count() == 0 {
            return Ok(());
        }
        let merged = target_a
            .core()
            .merge_with_default(target_b.core(), &self.default, |a, b| {
                P::resolve(a.clone(), b.clone())
            });
        *target_a = WorkingTree::from_core(merged);
        Ok(())
    }

    fn inverse_bounds(
        &self,
        output_bounds: crate::RangeId,
    ) -> (
        alloc::vec::Vec<crate::RangeId>,
        alloc::vec::Vec<crate::RangeId>,
    ) {
        (
            alloc::vec![output_bounds.clone()],
            alloc::vec![output_bounds],
        )
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "merge({})", P::NAME)
    }
}
