use alloc::boxed::Box;

use crate::{
    Error, SpatialIdMap,
    spatial_id::collection::{
        flex_tree::core::SafeValue,
        query::{BinaryOperator, MergePolicy, ValueIter, cancellation::CancellationToken},
    },
};

/// `MergePolicy` で重ね合わせる二項演算子。
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
    fn run<'a>(
        &'a self,
        lhs: ValueIter<'a, V>,
        rhs: ValueIter<'a, V>,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let lhs_tree: SpatialIdMap<V> = lhs.collect();
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let rhs_tree: SpatialIdMap<V> = rhs.collect();
        if lhs_tree.is_empty() && rhs_tree.is_empty() {
            return Ok(Box::new(core::iter::empty()));
        }
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        let merged = lhs_tree.merge_with_default(&rhs_tree, &self.default, |a, b| {
            P::resolve(a.clone(), b.clone())
        });
        Ok(Box::new(merged.into_iter()))
    }
}
