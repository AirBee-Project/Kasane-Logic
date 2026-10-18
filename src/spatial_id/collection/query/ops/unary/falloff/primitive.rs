use super::FalloffAxis;
use crate::{Error, FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl FlexId {
    pub(crate) fn falloff<Z: Into<u8>, V>(
        &self,
        axis: FalloffAxis,
        z: Z,
        radius: u32,
        direction: Option<crate::spatial_id::helpers::Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<alloc::vec::IntoIter<(FlexId, V)>, Error>
    where
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
        <V as TryFrom<u32>>::Error: Debug,
    {
        let z = ZoomLevel::new(z.into())?.get();
        let rad = radius as i32;

        let mut out = Vec::with_capacity((rad * 2 + 1) as usize);
        if rad == 0 {
            out.push((*self, value.clone()));
            return Ok(out.into_iter());
        }

        let min_d = if direction == Some(crate::spatial_id::helpers::Side::Upper) {
            0
        } else {
            -rad
        };
        let max_d = if direction == Some(crate::spatial_id::helpers::Side::Lower) {
            0
        } else {
            rad
        };
        let attenuator = super::Attenuator::new(radius, pattern);
        for d in min_d..=max_d {
            let attenuated = attenuator.attenuate(value, d.unsigned_abs());
            let mut push = |moved: FlexId| out.push((moved, attenuated.clone()));
            match axis {
                FalloffAxis::F => {
                    if let Ok(ids) = self.shift_f(z, d) {
                        ids.for_each(&mut push);
                    }
                }
                FalloffAxis::X => {
                    if let Ok(ids) = self.shift_x(z, d) {
                        ids.for_each(&mut push);
                    }
                }
                FalloffAxis::Y => {
                    if let Ok(ids) = self.shift_y(z, d) {
                        ids.for_each(&mut push);
                    }
                }
            }
        }

        Ok(out.into_iter())
    }

    /// F方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_f<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<crate::spatial_id::helpers::Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
        <V as TryFrom<u32>>::Error: Debug,
    {
        self.falloff(FalloffAxis::F, z, radius, direction, pattern, value)
    }

    /// X方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_x<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<crate::spatial_id::helpers::Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
        <V as TryFrom<u32>>::Error: Debug,
    {
        self.falloff(FalloffAxis::X, z, radius, direction, pattern, value)
    }

    /// Y方向へ値を減少させる。指定した距離で0になる。
    pub fn falloff_y<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        direction: Option<crate::spatial_id::helpers::Side>,
        pattern: super::FalloffPattern,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)> + use<Z, V>, Error>
    where
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
        <V as TryFrom<u32>>::Error: Debug,
    {
        self.falloff(FalloffAxis::Y, z, radius, direction, pattern, value)
    }
}
