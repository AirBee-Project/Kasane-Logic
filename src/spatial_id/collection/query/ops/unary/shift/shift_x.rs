use crate::SpatialIdTable;
use crate::spatial_id::collection::flex_tree::core::NoSummary;
use alloc::boxed::Box;

use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::{
    Error, ZoomLevel,
    spatial_id::collection::query::traits::{QueryIter, UnaryOperator},
};

/// 作業木全体を東西（X）方向へ、ズームレベル `z` のインデックス値 `x` 個分だけ平行移動する単項演算。
pub struct ShiftX {
    z: ZoomLevel,
    x: i32,
}

impl ShiftX {
    /// ズーム `z` のSegment `x` 個分の東西移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, x: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, x })
    }
}

impl<V: SafeValue + 'static> UnaryOperator<V> for ShiftX {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_x(self.x.unsigned_abs())?;
        Ok(())
    }

    fn run<'a>(&self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.x;
        if index == 0 {
            return Ok(input);
        }

        let target: SpatialIdTable<V, NoSummary> = input.collect::<Result<_, _>>()?;
        let mut rebuilt = SpatialIdTable::<V, NoSummary>::default();
        for (id, value) in target.iter() {
            rebuilt.extend(id.shift_x(z, index)?.map(|moved| (moved, value.clone())));
        }
        Ok(Box::new(rebuilt.into_iter().map(Ok)))
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.x as i64) * (1i64 << (target_z - z));

        bounds.x_edges_shift(target_z, -delta, -delta).unwrap()
    }
}
