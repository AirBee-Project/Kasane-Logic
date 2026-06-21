use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::{Stretch, StretchParam};

pub trait StretchOps: SpatialIdCollection {
    /// 高さ（F）方向へ引き延ばす（衝突は後勝ち）。
    fn stretch_f(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_f_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 東西（X）方向へ引き延ばす（巡回・衝突は後勝ち）。
    fn stretch_x(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_x_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 南北（Y）方向へ引き延ばす（範囲外はエラー・衝突は後勝ち）。
    fn stretch_y(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_y_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 高さ（F）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_f_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Stretch::execution::<Self, Self>(self, StretchParam::f(z, index, conflict))
    }

    /// 東西（X）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_x_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Stretch::execution::<Self, Self>(self, StretchParam::x(z, index, conflict))
    }

    /// 南北（Y）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_y_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Stretch::execution::<Self, Self>(self, StretchParam::y(z, index, conflict))
    }
}

impl<C> StretchOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn stretch_f(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Stretch(StretchParam::f(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_x(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Stretch(StretchParam::x(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_y(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Plan::Unary(
            crate::spatial_id::collection::expr::plan::UnaryOp::Stretch(StretchParam::y(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}
