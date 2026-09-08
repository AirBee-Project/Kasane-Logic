use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{UnaryOperator, ValueIter};
use crate::{Error, RangeId, ZoomLevel};

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

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        _target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.x;
        if index == 0 {
            return Ok(input);
        }
        let mut counter = 0u32;
        Ok(Box::new(
            input
                .map_while(move |item| token.check_amortized(&mut counter).ok().map(|_| item))
                .flat_map(move |(id, value)| {
                    id.shift_x(z, index)
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
        let delta = (self.x as i64) * (1i64 << (target_z - z));
        bounds.x_edges_shift(target_z, -delta, -delta).unwrap()
    }

    fn forward_bounds(&self, bounds: RangeId) -> Option<RangeId> {
        // 単純な平行移動なので、逆算(-delta)の符号を反転させるだけでよい。
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.x as i64) * (1i64 << (target_z - z));
        bounds.x_edges_shift(target_z, delta, delta).unwrap()
    }
}
