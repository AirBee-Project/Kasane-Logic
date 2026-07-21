use super::{shift_f::ShiftF, shift_x::ShiftX, shift_y::ShiftY};
use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// F方向のShift演算を適用する
    pub fn shift_f<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftF::new(z, index) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// X方向のShift演算を適用する
    pub fn shift_x<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftX::new(z, index) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向のShift演算を適用する
    pub fn shift_y<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftY::new(z, index) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }
}
