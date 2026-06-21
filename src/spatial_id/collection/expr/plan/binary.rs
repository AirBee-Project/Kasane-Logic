use alloc::boxed::Box;

use crate::{ConflictPolicy, Error, SpatialIdCollection};

pub trait BinaryKernel<C: SpatialIdCollection> {
    fn run(self: Box<Self>, lhs: &C, rhs: &C) -> Result<C, Error>;
}

/// 二項項演算を「値」として列挙したもの。
///
/// 最適化の余地があるものは個別に実装し、それ以外のものはCustomとして実装する
pub enum BinaryOp<C: SpatialIdCollection> {
    Union(ConflictPolicy<C::Value>),
    Intersection(ConflictPolicy<C::Value>),
    Difference,
    SymmetricDifference,
    Mask,

    // 書かれていない演算子はここで吸収される
    Custom(Box<dyn BinaryKernel<C>>),
}

impl<C: SpatialIdCollection> BinaryOp<C> {
    pub fn run(self, lhs: &C, rhs: &C) -> Result<C, Error> {
        match self {
            BinaryOp::Union(param) => <crate::spatial_id::collection::expr::binary::set::union::Union as crate::BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, param),
            BinaryOp::Intersection(param) => <crate::spatial_id::collection::expr::binary::set::intersection::Intersection as crate::BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, param),
            BinaryOp::Difference => <crate::spatial_id::collection::expr::binary::set::difference::Difference as crate::BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, ()),
            BinaryOp::SymmetricDifference => <crate::spatial_id::collection::expr::binary::set::symmetric_difference::SymmetricDifference as crate::BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, ()),
            BinaryOp::Mask => <crate::spatial_id::collection::expr::binary::set::mask::Mask as crate::BinaryOperator<C::Value, C::Value>>::execution::<C, C, C>(lhs, rhs, ()),
            BinaryOp::Custom(kernel) => kernel.run(lhs, rhs),
        }
    }
}
