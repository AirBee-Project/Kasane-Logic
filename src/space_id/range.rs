use std::fmt;

use crate::{
    error::Error,
    space_id::constants::{F_MAX, F_MIN, XY_MAX},
};

pub struct RangeID {
    z: u8,
    f: [i64; 2],
    x: [u64; 2],
    y: [u64; 2],
}

impl fmt::Display for RangeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.z,
            format_dimension(self.f),
            format_dimension(self.x),
            format_dimension(self.y),
        )
    }
}

fn format_dimension<T: PartialEq + fmt::Display>(dimension: [T; 2]) -> String {
    if dimension[0] == dimension[1] {
        format!("{}", dimension[0])
    } else {
        format!("{}:{}", dimension[0], dimension[1])
    }
}

impl RangeID {
    pub fn new(z: u8, f: [i64; 2], x: [u64; 2], y: [u64; 2]) -> Result<RangeID, Error> {
        //ズームレベルが範囲内であることを検証する
        if z > 63 as u8 {
            return Err(Error::ZoomLevelOutOfRange { zoom_level: z });
        };

        //各次元の範囲を定数配列から読み込む
        let f_max = F_MAX[z as usize];
        let f_min = F_MIN[z as usize];
        let xy_max = XY_MAX[z as usize];

        todo!()
    }

    pub fn as_z(&self) -> &u8 {
        &self.z
    }

    pub fn as_f(&self) -> &[i64; 2] {
        &self.f
    }

    pub fn as_x(&self) -> &[u64; 2] {
        &self.x
    }

    pub fn as_y(&self) -> &[u64; 2] {
        &self.y
    }
}
