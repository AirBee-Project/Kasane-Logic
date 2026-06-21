use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::ShiftParam;
use super::shift_f::FShift;
use super::shift_x::XShift;
use super::shift_y::YShift;

pub trait ShiftOps: SpatialIdCollection {
    fn shift_f(&self, z: u8, index: i32) -> Result<Self, Error> {
        FShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }

    fn shift_x(&self, z: u8, index: i32) -> Result<Self, Error> {
        XShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }

    fn shift_y(&self, z: u8, index: i32) -> Result<Self, Error> {
        YShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }
}

impl<C> ShiftOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn shift_f(self, z: u8, index: i32) -> Self {
        self.apply_unary::<FShift>(ShiftParam { z, index })
    }

    pub fn shift_x(self, z: u8, index: i32) -> Self {
        self.apply_unary::<XShift>(ShiftParam { z, index })
    }

    pub fn shift_y(self, z: u8, index: i32) -> Self {
        self.apply_unary::<YShift>(ShiftParam { z, index })
    }
}
