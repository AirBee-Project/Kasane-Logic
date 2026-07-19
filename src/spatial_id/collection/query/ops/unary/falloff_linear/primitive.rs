use crate::{Error, FlexId, ZoomLevel};
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

impl FlexId {
    /// F方向へ指定した半径(radius)の範囲で、元の値(value)を減衰しながら伝播するIDと値のペアを返す。
    pub fn falloff_linear_f<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)>, Error>
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

    /// X方向へ指定した半径(radius)の範囲で、元の値(value)を減衰しながら伝播するIDと値のペアを返す。
    pub fn falloff_linear_x<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)>, Error>
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

    /// Y方向へ指定した半径(radius)の範囲で、元の値(value)を減衰しながら伝播するIDと値のペアを返す。
    pub fn falloff_linear_y<Z: Into<u8>, V>(
        &self,
        z: Z,
        radius: u32,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)>, Error>
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

    /// F, X, Y方向へ直方体状に減衰しながら元の値(value)を伝播するIDと値のペアを返す。
    pub fn falloff_linear_fxy<Z: Into<u8>, V>(
        &self,
        z: Z,
        f_radius: u32,
        x_radius: u32,
        y_radius: u32,
        value: &V,
    ) -> Result<impl Iterator<Item = (FlexId, V)>, Error>
    where
        V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
        <V as TryFrom<u32>>::Error: Debug,
    {
        let z = ZoomLevel::new(z.into())?.get();
        let f_rad = f_radius as i32;
        let x_rad = x_radius as i32;
        let y_rad = y_radius as i32;

        let est_cap = (f_rad * 2 + 1) * (x_rad * 2 + 1) * (y_rad * 2 + 1);
        let mut out = Vec::with_capacity(est_cap as usize);
        if f_rad == 0 && x_rad == 0 && y_rad == 0 {
            out.push((self.clone(), value.clone()));
            return Ok(out.into_iter());
        }

        // Hoist buffers to avoid inner loop allocations
        let mut current_ids = Vec::with_capacity(8);
        let mut next_ids = Vec::with_capacity(8);

        for df in -f_rad..=f_rad {
            for dx in -x_rad..=x_rad {
                for dy in -y_rad..=y_rad {
                    let df_num = df.unsigned_abs();
                    let dx_num = dx.unsigned_abs();
                    let dy_num = dy.unsigned_abs();

                    let mut max_num = 0;
                    let mut max_den = 1;

                    if f_radius > 0
                        && df_num as u64 * max_den as u64 > max_num as u64 * f_radius as u64
                    {
                        max_num = df_num;
                        max_den = f_radius;
                    }
                    if x_radius > 0
                        && dx_num as u64 * max_den as u64 > max_num as u64 * x_radius as u64
                    {
                        max_num = dx_num;
                        max_den = x_radius;
                    }
                    if y_radius > 0
                        && dy_num as u64 * max_den as u64 > max_num as u64 * y_radius as u64
                    {
                        max_num = dy_num;
                        max_den = y_radius;
                    }

                    if max_num > max_den {
                        continue;
                    }

                    let v_distance = V::try_from(max_num).unwrap();
                    let v_radius = V::try_from(max_den).unwrap();
                    let attenuated =
                        (value.clone() * (v_radius - v_distance)) / V::try_from(max_den).unwrap();

                    current_ids.clear();
                    current_ids.push(self.clone());

                    if df != 0 {
                        next_ids.clear();
                        for i in &current_ids {
                            if let Ok(moved) = i.shift_f(z, df) {
                                next_ids.extend(moved);
                            }
                        }
                        core::mem::swap(&mut current_ids, &mut next_ids);
                    }
                    if dx != 0 {
                        next_ids.clear();
                        for i in &current_ids {
                            if let Ok(moved) = i.shift_x(z, dx) {
                                next_ids.extend(moved);
                            }
                        }
                        core::mem::swap(&mut current_ids, &mut next_ids);
                    }
                    if dy != 0 {
                        next_ids.clear();
                        for i in &current_ids {
                            if let Ok(moved) = i.shift_y(z, dy) {
                                next_ids.extend(moved);
                            }
                        }
                        core::mem::swap(&mut current_ids, &mut next_ids);
                    }

                    for moved in current_ids.drain(..) {
                        out.push((moved, attenuated.clone()));
                    }
                }
            }
        }

        Ok(out.into_iter())
    }
}
