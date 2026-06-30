use crate::{ConflictPolicy, SpatialIdCollection};

use super::StretchParam;

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn stretch_f(self, z: u8, index: i32) -> Self {
        self.stretch_f_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_x(self, z: u8, index: i32) -> Self {
        self.stretch_x_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_y(self, z: u8, index: i32) -> Self {
        self.stretch_y_with(z, index, ConflictPolicy::Overwrite)
    }

    pub fn stretch_f_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Stretch(StretchParam::f(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_x_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Stretch(StretchParam::x(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn stretch_y_with(self, z: u8, index: i32, conflict: ConflictPolicy<C::Value>) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Stretch(StretchParam::y(
                z, index, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }
}
