use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::StretchParam;
use super::stretch_f::FStretch;
use super::stretch_x::XStretch;
use super::stretch_y::YStretch;

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
        FStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }

    /// 東西（X）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_x_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        XStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }

    /// 南北（Y）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_y_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        YStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }
}

impl<C> StretchOps for C where C: SpatialIdCollection {}

use crate::spatial_id::collection::expr::plan::Plan;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    pub fn stretch_f(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        self.apply_unary::<FStretch>(StretchParam { z, index, conflict })
    }

    pub fn stretch_x(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        self.apply_unary::<XStretch>(StretchParam { z, index, conflict })
    }

    pub fn stretch_y(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        self.apply_unary::<YStretch>(StretchParam { z, index, conflict })
    }
}
