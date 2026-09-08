use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::{UnaryOperator, ValueIter};
use crate::{Error, RangeId, ZoomLevel};

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

    fn run<'a>(
        &'a self,
        input: ValueIter<'a, V>,
        _target: RangeId,
        token: CancellationToken,
    ) -> Result<ValueIter<'a, V>, Error> {
        let z = self.z.get();
        let index = self.f;
        if index == 0 {
            return Ok(input);
        }
        let mut counter = 0u32;
        Ok(Box::new(
            input
                .map_while(move |item| token.check_amortized(&mut counter).ok().map(|_| item))
                .flat_map(move |(id, value)| {
                    // F方向は周回しないため、上下端付近のSegmentは移動先がズーム範囲外になり得る。
                    // 個々の要素をResultにして全段を貫通させるコストを避け、範囲外に出た要素は消える。
                    id.shift_f(z, index)
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
        let delta = (self.f as i64) * (1i64 << (target_z - z));
        bounds.f_edges_shift(target_z, -delta, -delta).unwrap()
    }
}
