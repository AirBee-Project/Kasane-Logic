use crate::spatial_id::collection::expr::plan::binary::kernel::BinaryKernel;
use crate::{BinaryOperator, ConflictPolicy, Error, SpatialIdCollection};

pub enum BinaryOp<C: SpatialIdCollection> {
    Union(ConflictPolicy<C::Value>),
    Intersection(ConflictPolicy<C::Value>),
    Difference,
    SymmetricDifference,
    Mask,
    Custom(alloc::boxed::Box<dyn BinaryKernel<C>>),
}

impl<C: SpatialIdCollection> BinaryOp<C> {
    /// この演算が可換かどうか。
    pub fn is_commutative(&self) -> bool {
        use crate::spatial_id::collection::expr::binary::set::{
            difference::Difference, intersection::Intersection, mask::Mask,
            symmetric_difference::SymmetricDifference, union::Union,
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

    pub fn run(self, lhs: &C, rhs: &C) -> Result<C, Error> {
        match self {
            BinaryOp::Union(p) => {
                crate::spatial_id::collection::expr::binary::set::union::Union::execution::<C, C, C>(
                    lhs, rhs, p,
                )
            }
            BinaryOp::Intersection(p) => {
                crate::spatial_id::collection::expr::binary::set::intersection::Intersection::execution::<
                    C,
                    C,
                    C,
                >(lhs, rhs, p)
            }
            BinaryOp::Difference => {
                crate::spatial_id::collection::expr::binary::set::difference::Difference::execution::<
                    C,
                    C,
                    C,
                >(lhs, rhs, ())
            }
            BinaryOp::SymmetricDifference => {
                crate::spatial_id::collection::expr::binary::set::symmetric_difference::SymmetricDifference::execution::<
                    C,
                    C,
                    C,
                >(lhs, rhs, ())
            }
            BinaryOp::Mask => {
                crate::spatial_id::collection::expr::binary::set::mask::Mask::execution::<C, C, C>(
                    lhs, rhs, (),
                )
            }
            BinaryOp::Custom(kernel) => kernel.run(lhs, rhs),
        }
    }
}
