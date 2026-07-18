use super::{shift_f::ShiftF, shift_fxy::ShiftFXY, shift_x::ShiftX, shift_y::ShiftY};
use crate::{Error, SpatialIdCollection, spatial_id::collection::query::execution::Query};
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
    ) -> Result<Self, Error> {
        let op = ShiftFXY::new(f, x, y)?;
        Ok(Query::Unary(Box::new(op), Box::new(self)))
    }

    /// F方向のShift演算を適用する
    pub fn shift_f<Z: Into<u8>>(self, z: Z, index: i32) -> Result<Self, Error> {
        let op = ShiftF::new(z, index)?;
        Ok(Query::Unary(Box::new(op), Box::new(self)))
    }

    /// X方向のShift演算を適用する
    pub fn shift_x<Z: Into<u8>>(self, z: Z, index: i32) -> Result<Self, Error> {
        let op = ShiftX::new(z, index)?;
        Ok(Query::Unary(Box::new(op), Box::new(self)))
    }

    /// Y方向のShift演算を適用する
    pub fn shift_y<Z: Into<u8>>(self, z: Z, index: i32) -> Result<Self, Error> {
        let op = ShiftY::new(z, index)?;
        Ok(Query::Unary(Box::new(op), Box::new(self)))
    }
}
