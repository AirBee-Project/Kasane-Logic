use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::{Level, LevelParam};

pub trait LevelOps: SpatialIdCollection {
    fn level_f(&self, z: u8, lo: i32, hi: i32) -> Result<Self, Error> {
        self.level_f_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_x(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_x_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_y(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_y_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    fn level_f_with(
        &self,
        z: u8,
        lo: i32,
        hi: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Level::execution::<Self, Self>(self, LevelParam::f(z, lo, hi, conflict))
    }

    fn level_x_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Level::execution::<Self, Self>(self, LevelParam::x(z, lo, hi, conflict))
    }

    fn level_y_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Level::execution::<Self, Self>(self, LevelParam::y(z, lo, hi, conflict))
    }
}

impl<C> LevelOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn level_f(self, z: u8, lo: i32, hi: i32) -> Self {
        self.level_f_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_x(self, z: u8, lo: u32, hi: u32) -> Self {
        self.level_x_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_y(self, z: u8, lo: u32, hi: u32) -> Self {
        self.level_y_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    pub fn level_f_with(self, z: u8, lo: i32, hi: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Level(LevelParam::f(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_x_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Level(LevelParam::x(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_y_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Level(LevelParam::y(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}
