use super::Shift;
use crate::spatial_id::collection::query::Query;
use crate::spatial_id::collection::query::send_sync::SafeValue;

impl<V: SafeValue + 'static> Query<V> {
    /// ズームレベル `z` のインデックス値で、F・X・Y 方向へそれぞれ `f`・`x`・`y` 個分だけ平行移動する。
    pub fn shift_fxy<Z: Into<u8>>(self, z: Z, f: i32, x: i32, y: i32) -> Self {
        if matches!(self, Query::Error(_)) {
            return self;
        }
        match Shift::new(z, f, x, y) {
            Ok(op) => self.wrap_unary(op),
            Err(e) => Query::Error(e),
        }
    }

    /// F方向のShift演算を適用する
    pub fn shift_f<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        self.shift_fxy(z, index, 0, 0)
    }

    /// X方向のShift演算を適用する
    pub fn shift_x<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        self.shift_fxy(z, 0, index, 0)
    }

    /// Y方向のShift演算を適用する
    pub fn shift_y<Z: Into<u8>>(self, z: Z, index: i32) -> Self {
        self.shift_fxy(z, 0, 0, index)
    }
}
