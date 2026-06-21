use crate::{BinaryOperator, ConflictPolicy, Error, SpatialIdCollection};

use super::combine::Combine;
use super::difference::Difference;
use super::intersection::Intersection;
use super::mask::Mask;
use super::symmetric_difference::SymmetricDifference;
use super::union::Union;

pub trait SetOps: SpatialIdCollection {
    fn union_with(&self, other: &Self, policy: ConflictPolicy<Self::Value>) -> Result<Self, Error> {
        Union::execution::<Self, Self, Self>(self, other, policy)
    }

    fn intersection_with(
        &self,
        other: &Self,
        policy: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Intersection::execution::<Self, Self, Self>(self, other, policy)
    }

    fn symmetric_difference(&self, other: &Self) -> Result<Self, Error> {
        SymmetricDifference::execution::<Self, Self, Self>(self, other, ())
    }

    fn difference<B>(&self, other: &B) -> Result<Self, Error>
    where
        B: SpatialIdCollection,
    {
        Difference::execution::<Self, B, Self>(self, other, ())
    }

    fn mask<B>(&self, other: &B) -> Result<Self, Error>
    where
        B: SpatialIdCollection,
    {
        Mask::execution::<Self, B, Self>(self, other, ())
    }

    fn combine_with<B, R, F>(&self, other: &B, f: F) -> Result<R, Error>
    where
        B: SpatialIdCollection,
        R: SpatialIdCollection,
        F: Fn(Option<&Self::Value>, Option<&B::Value>) -> Option<R::Value>,
    {
        Combine::<F, R::Value>::execution::<Self, B, R>(self, other, f)
    }
}

impl<C> SetOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn union_with(self, other: Self, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Binary(
            crate::spatial_id::collection::expr::plan::BinaryOp::Union(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other),
        )
    }

    pub fn intersection_with(self, other: Self, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Binary(
            crate::spatial_id::collection::expr::plan::BinaryOp::Intersection(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other),
        )
    }

    pub fn difference(self, other: Self) -> Self {
        Plan::Binary(
            crate::spatial_id::collection::expr::plan::BinaryOp::Difference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other),
        )
    }

    pub fn symmetric_difference(self, other: Self) -> Self {
        Plan::Binary(
            crate::spatial_id::collection::expr::plan::BinaryOp::SymmetricDifference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other),
        )
    }

    pub fn mask(self, other: Self) -> Self {
        Plan::Binary(
            crate::spatial_id::collection::expr::plan::BinaryOp::Mask,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other),
        )
    }
}
