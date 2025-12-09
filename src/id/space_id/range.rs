use std::fmt;

use itertools::iproduct;

use crate::{
    bit_vec::BitVec,
    id::encode::EncodeID,
    error::Error,
    id::space_id::segment::Segment,
    id::space_id::{
        SpaceId,
        constants::{F_MAX, F_MIN, XY_MAX},
        single::SingleID,
    },
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
        if z > 63u8 {
            return Err(Error::ZoomLevelOutOfRange { zoom_level: z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        // fの範囲が逆転していないか
        if f[0] > f[1] {
            return Err(Error::FRangeReversed {
                start: f[0],
                end: f[1],
            });
        }

        // fの範囲チェック
        if f[0] < f_min || f[0] > f_max {
            return Err(Error::FOutOfRange { f: f[0], z });
        }
        if f[1] < f_min || f[1] > f_max {
            return Err(Error::FOutOfRange { f: f[1], z });
        }

        // xの範囲チェック
        if x[0] > xy_max {
            return Err(Error::XOutOfRange { x: x[0], z });
        }
        if x[1] > xy_max {
            return Err(Error::XOutOfRange { x: x[1], z });
        }

        // yの範囲チェック
        if y[0] > xy_max {
            return Err(Error::YOutOfRange { y: y[0], z });
        }
        if y[1] > xy_max {
            return Err(Error::YOutOfRange { y: y[1], z });
        }

        Ok(RangeID { z, f, x, y })
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

    pub fn children(&self, difference: u8) -> Result<RangeID, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZoomLevelOutOfRange {
                zoom_level: u8::MAX,
            })?;

        if z > 63 {
            return Err(Error::ZoomLevelOutOfRange { zoom_level: z });
        }

        let scale_f = 2_i64.pow(difference as u32);
        let scale_xy = 2_u64.pow(difference as u32);

        let f = [self.f[0] * scale_f, self.f[1] * scale_f + scale_f - 1];
        let x = [self.x[0] * scale_xy, self.x[1] * scale_xy + scale_xy - 1];
        let y = [self.y[0] * scale_xy, self.y[1] * scale_xy + scale_xy - 1];

        Ok(RangeID { z, f, x, y })
    }

    pub fn parent(&self, difference: u8) -> Option<RangeID> {
        let z = self.z.checked_sub(difference)?;
        let shift = difference as u32;

        let f = [
            if self.f[0] == -1 {
                -1
            } else {
                self.f[0] >> shift
            },
            if self.f[1] == -1 {
                -1
            } else {
                self.f[1] >> shift
            },
        ];

        let x = [self.x[0] >> shift, self.x[1] >> shift];
        let y = [self.y[0] >> shift, self.y[1] >> shift];

        Some(RangeID { z, f, x, y })
    }

    pub fn to_single_id(&self) -> impl Iterator<Item = SingleID> + '_ {
        let f_range = self.f[0]..=self.f[1];
        let x_range = self.x[0]..=self.x[1];
        let y_range = self.y[0]..=self.y[1];

        iproduct!(f_range, x_range, y_range).map(move |(f, x, y)| SingleID { z: self.z, f, x, y })
    }
}

impl SpaceId for RangeID {
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }

    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }

    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    fn move_up(&mut self, by: i64) -> Result<(), Error> {
        let new_start = self.f[0].checked_add(by).ok_or(Error::FOutOfRange {
            f: i64::MAX,
            z: self.z,
        })?;
        let new_end = self.f[1].checked_add(by).ok_or(Error::FOutOfRange {
            f: i64::MAX,
            z: self.z,
        })?;

        if new_start < self.min_f() || new_end > self.max_f() {
            return Err(Error::FOutOfRange {
                f: new_start.max(new_end),
                z: self.z,
            });
        }

        self.f[0] = new_start;
        self.f[1] = new_end;
        Ok(())
    }

    fn move_down(&mut self, by: i64) -> Result<(), Error> {
        let new_start = self.f[0].checked_sub(by).ok_or(Error::FOutOfRange {
            f: i64::MIN,
            z: self.z,
        })?;
        let new_end = self.f[1].checked_sub(by).ok_or(Error::FOutOfRange {
            f: i64::MIN,
            z: self.z,
        })?;

        if new_start < self.min_f() || new_end > self.max_f() {
            return Err(Error::FOutOfRange {
                f: new_start.min(new_end),
                z: self.z,
            });
        }

        self.f[0] = new_start;
        self.f[1] = new_end;
        Ok(())
    }

    fn move_north(&mut self, by: u64) {
        self.y[0] = (self.y[0].wrapping_add(by)) % self.max_xy();
        self.y[1] = (self.y[1].wrapping_add(by)) % self.max_xy();
    }

    fn move_south(&mut self, by: u64) {
        self.y[0] = (self.y[0].wrapping_sub(by)) % self.max_xy();
        self.y[1] = (self.y[1].wrapping_sub(by)) % self.max_xy();
    }

    fn move_east(&mut self, by: u64) {
        self.x[0] = (self.x[0].wrapping_add(by)) % self.max_xy();
        self.x[1] = (self.x[1].wrapping_add(by)) % self.max_xy();
    }

    fn move_west(&mut self, by: u64) {
        self.x[0] = (self.x[0].wrapping_sub(by)) % self.max_xy();
        self.x[1] = (self.x[1].wrapping_sub(by)) % self.max_xy();
    }
}

impl From<RangeID> for EncodeID {
    fn from(id: RangeID) -> Self {
        let f_segment = Segment::<i64>::new(id.z, id.f);
        let x_segment = Segment::<u64>::new(id.z, id.x);
        let y_segment = Segment::<u64>::new(id.z, id.y);

        let f_bitvec: Vec<BitVec> = f_segment.iter().map(|f| f.to_bitvec()).collect();
        let x_bitvec: Vec<BitVec> = x_segment.iter().map(|x| x.to_bitvec()).collect();
        let y_bitvec: Vec<BitVec> = y_segment.iter().map(|y| y.to_bitvec()).collect();

        EncodeID {
            f: f_bitvec,
            x: x_bitvec,
            y: y_bitvec,
        }
    }
}
