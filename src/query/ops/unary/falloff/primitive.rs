use super::{Attenuator, FalloffAxis};
use crate::id::helpers::Side;
use crate::{Error, FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::ops::{Add, Div, Mul, Rem, Sub};

impl FlexId {
    pub(crate) fn falloff<Z: Into<u8>, V>(
        &self,
        axis: FalloffAxis,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<alloc::vec::IntoIter<(FlexId, V)>, Error>
    where
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone,
    {
        let zoom = ZoomLevel::new(z.into())?;
        axis.check_radius(zoom, radius)?;
        let attenuator = Attenuator::new(radius, pattern)?;
        let mut out = Vec::new();
        self.falloff_each(
            axis,
            zoom,
            radius,
            direction,
            &attenuator,
            value,
            &mut Vec::new(),
            |ids, value| out.extend(ids.iter().map(|&id| (id, value.clone()))),
        );
        Ok(out.into_iter())
    }

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

    /// F方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_f<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone,
    {
        self.falloff(FalloffAxis::F, z, radius, direction, pattern, value)
    }

    /// X方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_x<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone,
    {
        self.falloff(FalloffAxis::X, z, radius, direction, pattern, value)
    }

    /// Y方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_y<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Add<Output = V>
            + Sub<Output = V>
            + Mul<Output = V>
            + Div<Output = V>
            + Rem<Output = V>
            + TryFrom<u64>
            + Clone,
    {
        self.falloff(FalloffAxis::Y, z, radius, direction, pattern, value)
    }
}
