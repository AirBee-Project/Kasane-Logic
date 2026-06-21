use alloc::boxed::Box;

use crate::{
    Error, SpatialIdCollection, UnaryOperator,
    spatial_id::collection::expr::unary::{shift::ShiftParam, stretch::StretchParam},
};

/// 単項演算を「値」として列挙したもの。
///
/// 最適化の余地があるものは個別に実装し、それ以外のものはCustomとして実装する
pub enum UnaryOp<C: SpatialIdCollection> {
    ShiftF(ShiftParam),
    ShiftX(ShiftParam),
    ShiftY(ShiftParam),
    StretchF(StretchParam<C::Value>),
    StretchX(StretchParam<C::Value>),
    StretchY(StretchParam<C::Value>),
    Fill(C::Value),

    // 書かれていない演算子はここで吸収される
    Custom(Box<dyn UnaryKernel<C>>),
}

impl<C: SpatialIdCollection> UnaryOp<C> {
    pub fn run(self, input: &C) -> Result<C, Error> {
        match self {
            UnaryOp::ShiftF(param) => <crate::spatial_id::collection::expr::unary::shift::shift_f::FShift as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::ShiftX(param) => <crate::spatial_id::collection::expr::unary::shift::shift_x::XShift as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::ShiftY(param) => <crate::spatial_id::collection::expr::unary::shift::shift_y::YShift as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::StretchF(param) => <crate::spatial_id::collection::expr::unary::stretch::stretch_f::FStretch as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::StretchX(param) => <crate::spatial_id::collection::expr::unary::stretch::stretch_x::XStretch as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::StretchY(param) => <crate::spatial_id::collection::expr::unary::stretch::stretch_y::YStretch as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::Fill(param) => <crate::spatial_id::collection::expr::unary::fill::FillDefault as UnaryOperator<C::Value>>::execution::<C, C>(input, param),
            UnaryOp::Custom(kernel) => kernel.run(input),
        }
    }
}

pub trait UnaryKernel<C: SpatialIdCollection> {
    fn run(self: Box<Self>, input: &C) -> Result<C, Error>;
}
