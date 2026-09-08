use alloc::boxed::Box;

use crate::{
    Error, SpatialIdMap,
    spatial_id::collection::{
        flex_tree::core::SafeValue,
        query::{BinaryOperator, ValueIter, cancellation::CancellationToken},
    },
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
    fn run<'a>(
        &'a self,
        lhs: ValueIter<'a, V>,
        rhs: ValueIter<'a, V>,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let lhs_tree: SpatialIdMap<V> = lhs.collect();
        if lhs_tree.is_empty() {
            return Ok(Box::new(core::iter::empty()));
        }
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let rhs_tree: SpatialIdMap<V> = rhs.collect();
        if rhs_tree.is_empty() {
            return Ok(Box::new(lhs_tree.into_iter()));
        }
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        let diff = lhs_tree.difference(&rhs_tree);
        Ok(Box::new(diff.into_iter()))
    }
}
