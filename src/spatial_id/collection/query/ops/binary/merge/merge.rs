use crate::{
    Error,
    spatial_id::collection::query::{
        merge_policy::MergePolicy,
        traits::{BinaryOperator, WorkingTree},
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

impl<W, P> BinaryOperator<W> for Merge<W::Value, P>
where
    W: WorkingTree,
    P: MergePolicy<W::Value>,
{
    fn run(&self, target_a: &mut W, target_b: &W) -> Result<(), Error> {
        if target_a.count() == 0 && target_b.count() == 0 {
            return Ok(());
        }
        *target_a = target_a.merge_with_default(target_b, &self.default, |a, b| {
            P::resolve(a.clone(), b.clone())
        });
        Ok(())
    }

    fn inverse_bounds(&self, output_bounds: crate::RangeId) -> (alloc::vec::Vec<crate::RangeId>, alloc::vec::Vec<crate::RangeId>) {
        (alloc::vec![output_bounds.clone()], alloc::vec![output_bounds])
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "merge({})", P::NAME)
    }
}
