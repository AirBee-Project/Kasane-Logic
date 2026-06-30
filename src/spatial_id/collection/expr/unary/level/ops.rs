use crate::{ConflictPolicy, SpatialIdCollection};

use super::LevelParam;

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
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
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Level(LevelParam::f(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_x_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Level(LevelParam::x(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn level_y_with(self, z: u8, lo: u32, hi: u32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Level(LevelParam::y(
                z, lo, hi, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}
