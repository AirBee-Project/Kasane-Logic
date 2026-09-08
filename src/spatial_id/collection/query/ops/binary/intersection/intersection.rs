use alloc::boxed::Box;

use crate::{
    Error, SpatialIdMap,
    spatial_id::collection::{
        flex_tree::core::SafeValue,
        query::{BinaryOperator, ValueIter, cancellation::CancellationToken},
    },
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
            return Ok(Box::new(core::iter::empty()));
        }
        if token.is_cancelled() {
            return Err(Error::Cancelled);
        }

        // A ∩ B = A - (A - B)
        // これにより、Aの要素のValueを完全に維持しながら、AとBが重複する領域だけを残すことができます。
        let not_b = lhs_tree.difference(&rhs_tree);
        let intersection = lhs_tree.difference(&not_b);
        Ok(Box::new(intersection.into_iter()))
    }
}
