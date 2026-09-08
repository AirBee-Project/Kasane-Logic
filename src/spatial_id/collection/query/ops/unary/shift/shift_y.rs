use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{UnaryOperator, ValueIter};
use crate::{Error, RangeId, ZoomLevel};

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

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        _target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.y;
        if index == 0 {
            return Ok(input);
        }
        let mut counter = 0u32;
        Ok(Box::new(
            input
                .map_while(move |item| token.check_amortized(&mut counter).ok().map(|_| item))
                .flat_map(move |(id, value)| {
                    // Y方向は周回しないため、極付近のSegmentは移動先がズーム範囲外になり得る。
                    // 個々の要素をResultにして全段を貫通させるコストを避け、範囲外に出た要素は消える。
                    id.shift_y(z, index)
                        .ok()
                        .into_iter()
                        .flatten()
                        .map(move |moved| (moved, value.clone()))
                }),
        ))
    }

    fn inverse_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.y as i64) * (1i64 << (target_z - z));
        bounds.y_edges_shift(target_z, -delta, -delta).unwrap()
    }

    fn forward_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.y as i64) * (1i64 << (target_z - z));
        bounds.y_edges_shift(target_z, delta, delta).unwrap()
    }
}
