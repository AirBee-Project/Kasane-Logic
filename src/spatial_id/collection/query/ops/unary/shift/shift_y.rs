use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{QueryIter, UnaryOperator},
};

/// 作業木全体を南北（Y）方向へ、ズームレベル `z` のインデックス値 `y` 個分だけ平行移動する単項演算。
pub struct ShiftY {
    z: ZoomLevel,
    y: i32,
}

impl ShiftY {
    /// ズーム `z` のインデックス値 `y` 個分の南北移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, y: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, y })
    }
}

impl<V: SafeValue + 'static> UnaryOperator<V> for ShiftY {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_y(self.y.unsigned_abs())?;
        Ok(())
    }

    fn run<'a>(&self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.y;
        if index == 0 {
            return Ok(input);
        }

        super::shift_all(input, move |id| id.shift_y(z, index))
    }

    fn reverse_target(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.y as i64) * (1i64 << (target_z - z));

        bounds.y_edges_shift(target_z, -delta, -delta).unwrap()
    }
}
