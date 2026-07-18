use super::{shift_f::ShiftF, shift_fxy::ShiftFXY, shift_x::ShiftX, shift_y::ShiftY};
use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};
use alloc::boxed::Box;

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// 3次元統合Shift演算を適用する
    pub fn shift_fxy<T: Into<u8>, U: Into<u8>, V: Into<u8>>(
        self,
        f: (T, i32),
        x: (U, i32),
        y: (V, i32),
    ) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftFXY::new(f, x, y) {
            Ok(op) => Query::Unary(Box::new(op), Box::new(self)),
            Err(e) => Query::Error(e),
        }
    }

    /// F方向のShift演算を適用する
    pub fn shift_f<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftF::new(z, index) {
            Ok(op) => Query::Unary(Box::new(op), Box::new(self)),
            Err(e) => Query::Error(e),
        }
    }

    /// X方向のShift演算を適用する
    pub fn shift_x<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftX::new(z, index) {
            Ok(op) => Query::Unary(Box::new(op), Box::new(self)),
            Err(e) => Query::Error(e),
        }
    }

    /// Y方向のShift演算を適用する
    pub fn shift_y<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match ShiftY::new(z, index) {
            Ok(op) => Query::Unary(Box::new(op), Box::new(self)),
            Err(e) => Query::Error(e),
        }
    }
}
