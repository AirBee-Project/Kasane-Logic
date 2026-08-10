use crate::{Error, FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl FlexId {
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
        let z = ZoomLevel::new(z.into())?.get();
        let rad = radius as i32;

        let mut out = Vec::with_capacity((rad * 2 + 1) as usize);
        if rad == 0 {
            out.push((*self, value.clone()));
            return Ok(out.into_iter());
        }

        let min_df = if direction == Some(crate::spatial_id::helpers::Side::Upper) {
            0
        } else {
            -rad
        };
        let max_df = if direction == Some(crate::spatial_id::helpers::Side::Lower) {
            0
        } else {
            rad
        };
        let attenuator = super::Attenuator::new(radius, pattern);
        for df in min_df..=max_df {
            let attenuated = attenuator.attenuate(value, df.unsigned_abs());

            if let Ok(moved_ids) = self.shift_f(z, df) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
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
        let z = ZoomLevel::new(z.into())?.get();
        let rad = radius as i32;

        let mut out = Vec::with_capacity((rad * 2 + 1) as usize);
        if rad == 0 {
            out.push((*self, value.clone()));
            return Ok(out.into_iter());
        }

        let min_dx = if direction == Some(crate::spatial_id::helpers::Side::Upper) {
            0
        } else {
            -rad
        };
        let max_dx = if direction == Some(crate::spatial_id::helpers::Side::Lower) {
            0
        } else {
            rad
        };
        let attenuator = super::Attenuator::new(radius, pattern);
        for dx in min_dx..=max_dx {
            let attenuated = attenuator.attenuate(value, dx.unsigned_abs());

            if let Ok(moved_ids) = self.shift_x(z, dx) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
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
        let z = ZoomLevel::new(z.into())?.get();
        let rad = radius as i32;

        let mut out = Vec::with_capacity((rad * 2 + 1) as usize);
        if rad == 0 {
            out.push((*self, value.clone()));
            return Ok(out.into_iter());
        }

        let min_dy = if direction == Some(crate::spatial_id::helpers::Side::Upper) {
            0
        } else {
            -rad
        };
        let max_dy = if direction == Some(crate::spatial_id::helpers::Side::Lower) {
            0
        } else {
            rad
        };
        let attenuator = super::Attenuator::new(radius, pattern);
        for dy in min_dy..=max_dy {
            let attenuated = attenuator.attenuate(value, dy.unsigned_abs());

            if let Ok(moved_ids) = self.shift_y(z, dy) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
    }
}
