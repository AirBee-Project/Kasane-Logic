use crate::{ConflictPolicy, SpatialIdCollection};

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn union_with(
        self,
        other: impl Into<Query<C>>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Union(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn intersection_with(
        self,
        other: impl Into<Query<C>>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Intersection(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn difference(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Difference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn symmetric_difference(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::SymmetricDifference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn mask(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::expr::ops::binary::BinaryOp::Mask,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}
