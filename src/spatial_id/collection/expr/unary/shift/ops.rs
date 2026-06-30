use crate::SpatialIdCollection;

use super::ShiftParam;

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn shift_f(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Shift(ShiftParam::f(z, index)),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_x(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Shift(ShiftParam::x(z, index)),
            alloc::boxed::Box::new(self),
        )
    }

    pub fn shift_y(self, z: u8, index: i32) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Shift(ShiftParam::y(z, index)),
            alloc::boxed::Box::new(self),
        )
    }
}
