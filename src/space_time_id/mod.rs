use core::fmt;
use serde::Serialize;
pub mod dimension;
pub mod z_range;

use crate::{
    error::Error,
    space_time_id::{
        dimension::Dimension,
        z_range::{F_MAX, F_MIN, XY_MAX},
    },
};

impl fmt::Display for SpaceTimeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}_{}/{}",
            self.z, self.f, self.x, self.y, self.i, self.t
        )
    }
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceTimeId {
    pub z: u8,
    pub f: Dimension<i64>,
    pub x: Dimension<u64>,
    pub y: Dimension<u64>,
    pub i: u32,
    pub t: Dimension<u32>,
}

impl SpaceTimeId {
    /// 値の範囲を確認・正規化する
    pub fn new(
        z: u8,
        f: Dimension<i64>,
        x: Dimension<u64>,
        y: Dimension<u64>,
        i: u32,
        t: Dimension<u32>,
    ) -> Result<Self, Error> {
        if z > 60 {
            return Err(Error::ZoomLevelOutOfRange { zoom_level: z });
        }

        let f_max = F_MAX[z as usize];
        let f_min = F_MIN[z as usize];
        let xy_max = XY_MAX[z as usize];

        // 各軸の正規化
        let new_f = normalize_dimension(f, f_min, f_max, valid_range_f, z)?;
        let new_x = normalize_dimension(x, 0, xy_max, valid_range_x, z)?;
        let new_y = normalize_dimension(y, 0, xy_max, valid_range_y, z)?;

        let new_t: Dimension<u32> = Dimension::new(t.start, t.end);

        Ok(SpaceTimeId {
            z,
            f: new_f,
            x: new_x,
            y: new_y,
            i,
            t: new_t,
        })
    }
}

///次元の値が正しいかを判定するパッチ関数
fn normalize_dimension<T>(
    dim: Dimension<T>,
    min: T,
    max: T,
    validate: impl Fn(T, T, T, u8) -> Result<(), Error>,
    z: u8,
) -> Result<Dimension<T>, Error>
where
    T: PartialOrd + Copy,
{
    if let Some(s) = dim.start {
        validate(s, min, max, z)?;
    }
    if let Some(e) = dim.end {
        validate(e, min, max, z)?;
    }

    Ok(Dimension::new(dim.start, dim.end))
}

///Fの範囲が正しいかを確認する
fn valid_range_f(num: i64, min: i64, max: i64, z: u8) -> Result<(), Error> {
    if (min..=max).contains(&num) {
        Ok(())
    } else {
        Err(Error::FOutOfRange { f: num, z })
    }
}

///Xの範囲が正しいかを確認する
fn valid_range_x(num: u64, min: u64, max: u64, z: u8) -> Result<(), Error> {
    if (min..=max).contains(&num) {
        Ok(())
    } else {
        Err(Error::XOutOfRange { x: num, z })
    }
}

///Yの範囲が正しいかを確認する
fn valid_range_y(num: u64, min: u64, max: u64, z: u8) -> Result<(), Error> {
    if (min..=max).contains(&num) {
        Ok(())
    } else {
        Err(Error::YOutOfRange { y: num, z })
    }
}
