use crate::spatial_id::collection::query::ops::binary::kernel::BinaryKernel;
use crate::{BinaryOperator, ConflictPolicy, Error, SpatialIdCollection};

#[cfg(feature = "rayon")]
pub type DynBinaryKernel<C> = dyn BinaryKernel<C> + Send + Sync;
#[cfg(not(feature = "rayon"))]
pub type DynBinaryKernel<C> = dyn BinaryKernel<C>;

pub enum BinaryOp<C: SpatialIdCollection> {
    Union(ConflictPolicy<C::Value>),
    Intersection(ConflictPolicy<C::Value>),
    Difference,
    SymmetricDifference,
    Mask,
    Custom(alloc::boxed::Box<DynBinaryKernel<C>>),
}

impl<C: SpatialIdCollection> BinaryOp<C> {
    /// この演算が可換かどうか。
    pub fn is_commutative(&self) -> bool {
        use crate::spatial_id::collection::query::ops::binary::set::{
            Difference, Intersection, Mask, SymmetricDifference, Union,
        };

        match self {
            BinaryOp::Union(p) => <Union as BinaryOperator<C::Value, C::Value>>::is_commutative(p),
            BinaryOp::Intersection(p) => {
                <Intersection as BinaryOperator<C::Value, C::Value>>::is_commutative(p)
            }
            BinaryOp::Difference => {
                <Difference as BinaryOperator<C::Value, C::Value>>::is_commutative(&())
            }
            BinaryOp::SymmetricDifference => {
                <SymmetricDifference as BinaryOperator<C::Value, C::Value>>::is_commutative(&())
            }
            BinaryOp::Mask => <Mask as BinaryOperator<C::Value, C::Value>>::is_commutative(&()),
            BinaryOp::Custom(kernel) => kernel.is_commutative(),
        }
    }

    pub fn run(self, lhs: C, rhs: C) -> Result<C, Error> {
        match self {
            BinaryOp::Union(p) => {
                crate::spatial_id::collection::query::ops::binary::set::Union::execution::<C, C, C>(
                    lhs, rhs, p,
                )
            }
            BinaryOp::Intersection(p) => {
                crate::spatial_id::collection::query::ops::binary::set::Intersection::execution::<C, C, C>(lhs, rhs, p)
            }
            BinaryOp::Difference => {
                crate::spatial_id::collection::query::ops::binary::set::Difference::execution::<C, C, C>(lhs, rhs, ())
            }
            BinaryOp::SymmetricDifference => {
                crate::spatial_id::collection::query::ops::binary::set::SymmetricDifference::execution::<C, C, C>(lhs, rhs, ())
            }
            BinaryOp::Mask => {
                crate::spatial_id::collection::query::ops::binary::set::Mask::execution::<C, C, C>(
                    lhs, rhs, (),
                )
            }
            BinaryOp::Custom(kernel) => kernel.run(lhs, rhs),
        }
    }
}
