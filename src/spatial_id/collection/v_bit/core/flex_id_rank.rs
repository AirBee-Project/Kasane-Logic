use crate::{FlexId, MAX_ZOOM_LEVEL};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FlexIdRank {
    pub f_z: u8,
    pub x_z: u8,
    pub y_z: u8,
    pub f: i32,
    pub x: u32,
    pub y: u32,
}

impl FlexIdRank {
    #[inline]
    pub fn split(&self) -> (u128, u32) {
        let high_128 = ((self.f_z as u128) << 88)
            | ((self.x_z as u128) << 80)
            | ((self.y_z as u128) << 72)
            | (((self.f as u32) as u128) << 32)
            | (self.x as u128);

        let low_32 = self.y;
        (high_128, low_32)
    }

    #[inline]
    pub fn from_parts(high_128: u128, low_32: u32) -> Self {
        Self {
            f_z: (high_128 >> 88) as u8,
            x_z: (high_128 >> 80) as u8,
            y_z: (high_128 >> 72) as u8,
            f: (high_128 >> 32) as u32 as i32,
            x: (high_128 & 0xFFFFFFFF) as u32,
            y: low_32,
        }
    }
}

impl FlexId {
    pub fn flex_id_rank(&self) -> FlexIdRank {
        let (f_z, f_idx) = self.f();
        let (x_z, x_idx) = self.x();
        let (y_z, y_idx) = self.y();

        let scale_f = MAX_ZOOM_LEVEL as u8 - f_z;
        let scale_x = MAX_ZOOM_LEVEL as u8 - x_z;
        let scale_y = MAX_ZOOM_LEVEL as u8 - y_z;

        let f_start = f_idx << scale_f;
        let x_start = x_idx << scale_x;
        let y_start = y_idx << scale_y;

        let morton = morton_encode(x_start, y_start);

        FlexIdRank {
            f_z,
            x_z,
            y_z,
            f: f_start,
            x: (morton >> 32) as u32,
            y: (morton & 0xFFFFFFFF) as u32,
        }
    }
}

#[inline]
fn part1by1(mut n: u64) -> u64 {
    n &= 0x00000000ffffffff;
    n = (n | (n << 16)) & 0x0000FFFF0000FFFF;
    n = (n | (n << 8)) & 0x00FF00FF00FF00FF;
    n = (n | (n << 4)) & 0x0F0F0F0F0F0F0F0F;
    n = (n | (n << 2)) & 0x3333333333333333;
    n = (n | (n << 1)) & 0x5555555555555555;
    n
}

#[inline]
fn morton_encode(x: u32, y: u32) -> u64 {
    part1by1(x as u64) | (part1by1(y as u64) << 1)
}
