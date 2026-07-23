use crate::{Error, FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl FlexId {
    /// F方向のへ値をリニアに減少させる。指定した距離で0になる。
    pub fn falloff_linear_f<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
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
            out.push((self.clone(), value.clone()));
            return Ok(out.into_iter());
        }

        for df in -rad..=rad {
            let distance = df.unsigned_abs();
            let v_distance = V::try_from(distance).unwrap();
            let v_radius = V::try_from(radius).unwrap();
            let attenuated =
                (value.clone() * (v_radius - v_distance)) / V::try_from(radius).unwrap();

            if let Ok(moved_ids) = self.shift_f(z, df) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
    }

    /// F方向のへ値をリニアに減少させる。指定した距離で0になる。
    pub fn falloff_linear_x<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
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
            out.push((self.clone(), value.clone()));
            return Ok(out.into_iter());
        }

        for dx in -rad..=rad {
            let distance = dx.unsigned_abs();
            let v_distance = V::try_from(distance).unwrap();
            let v_radius = V::try_from(radius).unwrap();
            let attenuated =
                (value.clone() * (v_radius - v_distance)) / V::try_from(radius).unwrap();

            if let Ok(moved_ids) = self.shift_x(z, dx) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
    }

    /// F方向のへ値をリニアに減少させる。指定した距離で0になる。
    pub fn falloff_linear_y<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
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
            out.push((self.clone(), value.clone()));
            return Ok(out.into_iter());
        }

        for dy in -rad..=rad {
            let distance = dy.unsigned_abs();
            let v_distance = V::try_from(distance).unwrap();
            let v_radius = V::try_from(radius).unwrap();
            let attenuated =
                (value.clone() * (v_radius - v_distance)) / V::try_from(radius).unwrap();

            if let Ok(moved_ids) = self.shift_y(z, dy) {
                for moved in moved_ids {
                    out.push((moved, attenuated.clone()));
                }
            }
        }

        Ok(out.into_iter())
    }
}
