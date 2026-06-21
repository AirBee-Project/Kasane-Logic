use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::{Shift, ShiftParam};

pub trait ShiftOps: SpatialIdCollection {
    fn shift_f(&self, z: u8, index: i32) -> Result<Self, Error> {
        Shift::execution::<Self, Self>(self, ShiftParam::f(z, index))
    }

    fn shift_x(&self, z: u8, index: i32) -> Result<Self, Error> {
        Shift::execution::<Self, Self>(self, ShiftParam::x(z, index))
    }

    fn shift_y(&self, z: u8, index: i32) -> Result<Self, Error> {
        Shift::execution::<Self, Self>(self, ShiftParam::y(z, index))
    }
}

impl<C> ShiftOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn shift_f(self, z: u8, index: i32) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Shift(ShiftParam::f(z, index)),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_x(self, z: u8, index: i32) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Shift(ShiftParam::x(z, index)),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_y(self, z: u8, index: i32) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Shift(ShiftParam::y(z, index)),
            alloc::boxed::Box::new(self),
        )
    }
}
