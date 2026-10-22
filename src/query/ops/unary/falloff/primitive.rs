use super::{Attenuator, FalloffAxis};
use crate::id::helpers::Side;
use crate::{FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::ops::{Add, Div, Mul, Rem, Sub};

impl FlexId {
    /// 距離ごとに、移した先と減衰させた値を `emit` へ渡す。`radius` は `axis.check_radius` で確かめておく。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn falloff_each<V>(
        &self,
        axis: FalloffAxis,
        zoom: ZoomLevel,
        radius: u32,
        direction: Option<Side>,
        attenuator: &Attenuator<V>,
        value: &V,
        moved: &mut Vec<FlexId>,
        mut emit: impl FnMut(&[FlexId], V),
    ) where
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone,
    {
        if radius == 0 {
            emit(&[*self], value.clone());
            return;
        }
        let z = zoom.get();
        let rad = radius as i32;
        let min_d = if direction == Some(Side::Upper) {
            0
        } else {
            -rad
        };
        let max_d = if direction == Some(Side::Lower) {
            0
        } else {
            rad
        };
        for d in min_d..=max_d {
            moved.clear();
            // 軸の端を越える距離は移せないので、その距離の寄与は捨てる
            match axis {
                FalloffAxis::F => moved.extend(self.shift_f(z, d).into_iter().flatten()),
                FalloffAxis::X => moved.extend(self.shift_x(z, d).into_iter().flatten()),
                FalloffAxis::Y => moved.extend(self.shift_y(z, d).into_iter().flatten()),
            }
            if !moved.is_empty() {
                emit(moved, attenuator.attenuate(value, d.unsigned_abs()));
            }
        }
    }
}
