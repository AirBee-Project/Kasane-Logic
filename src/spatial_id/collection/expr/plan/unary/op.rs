use crate::spatial_id::collection::expr::plan::unary::kernel::UnaryKernel;
use crate::spatial_id::collection::expr::unary::level::LevelParam;
use crate::spatial_id::collection::expr::unary::shift::ShiftParam;
use crate::spatial_id::collection::expr::unary::stretch::StretchParam;
use crate::{Error, SpatialIdCollection, UnaryOperator};

pub enum UnaryOp<C: SpatialIdCollection> {
    ShiftF(ShiftParam),
    ShiftX(ShiftParam),
    ShiftY(ShiftParam),
    StretchF(StretchParam<C::Value>),
    StretchX(StretchParam<C::Value>),
    StretchY(StretchParam<C::Value>),
    LevelF(LevelParam<i32, C::Value>),
    LevelX(LevelParam<u32, C::Value>),
    LevelY(LevelParam<u32, C::Value>),
    Fill(C::Value),
    Custom(alloc::boxed::Box<dyn UnaryKernel<C>>),
}

impl<C: SpatialIdCollection> UnaryOp<C> {
    pub fn run(self, input: &C) -> Result<C, Error> {
        match self {
            UnaryOp::ShiftF(p) => {
                crate::spatial_id::collection::expr::unary::shift::shift_f::FShift::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::ShiftX(p) => {
                crate::spatial_id::collection::expr::unary::shift::shift_x::XShift::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::ShiftY(p) => {
                crate::spatial_id::collection::expr::unary::shift::shift_y::YShift::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::StretchF(p) => {
                crate::spatial_id::collection::expr::unary::stretch::stretch_f::FStretch::execution::<
                    C,
                    C,
                >(input, p)
            }
            UnaryOp::StretchX(p) => {
                crate::spatial_id::collection::expr::unary::stretch::stretch_x::XStretch::execution::<
                    C,
                    C,
                >(input, p)
            }
            UnaryOp::StretchY(p) => {
                crate::spatial_id::collection::expr::unary::stretch::stretch_y::YStretch::execution::<
                    C,
                    C,
                >(input, p)
            }
            UnaryOp::LevelF(p) => {
                crate::spatial_id::collection::expr::unary::level::level_f::FLevel::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::LevelX(p) => {
                crate::spatial_id::collection::expr::unary::level::level_x::XLevel::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::LevelY(p) => {
                crate::spatial_id::collection::expr::unary::level::level_y::YLevel::execution::<C, C>(
                    input, p,
                )
            }
            UnaryOp::Fill(v) => {
                crate::spatial_id::collection::expr::unary::fill::FillDefault::execution::<C, C>(
                    input, v,
                )
            }
            UnaryOp::Custom(kernel) => kernel.run(input),
        }
    }
}
