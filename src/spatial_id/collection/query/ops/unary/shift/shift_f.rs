use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{QueryIter, UnaryOperator},
};

/// 作業木全体を高さ（F）方向へ、ズームレベル `z` のインデックス値 `f` 個分だけ平行移動する単項演算。
pub struct ShiftF {
    z: ZoomLevel,
    f: i32,
}

impl ShiftF {
    /// ズーム `z` のインデックス値 `f` 個分の高さ移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, f: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, f })
    }
}

impl<V: SafeValue + 'static> UnaryOperator<V> for ShiftF {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_f(self.f)?;
        Ok(())
    }

    fn run<'a>(&self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.f;
        if index == 0 {
            return Ok(input);
        }

        Ok(super::shift_stream(input, move |id| id.shift_f(z, index)))
    }

    fn reverse_target(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.f as i64) * (1i64 << (target_z - z));

        bounds.f_edges_shift(target_z, -delta, -delta).unwrap()
    }
}
