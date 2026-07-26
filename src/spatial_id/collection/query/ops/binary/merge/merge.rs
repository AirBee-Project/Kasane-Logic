use crate::{
    Error, FlexTreeCore,
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
    fn run(&self, target_a: &mut FlexTreeCore<V>, target_b: &FlexTreeCore<V>) -> Result<(), Error> {
        if target_a.count() == 0 && target_b.count() == 0 {
            return Ok(());
        }
        *target_a = target_a.merge_with_default(target_b, &self.default, |a, b| {
            P::resolve(a.clone(), b.clone())
        });
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
