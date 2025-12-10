// src/id/space_id/range.rs
use itertools::iproduct;
use std::fmt;

use crate::{
    bit_vec::BitVec,
    error::Error,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, XY_MAX},
        encode::EncodeID,
        helpers,
        segment::Segment,
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
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f[0] > f[1] {
            return Err(Error::FRangeReversed {
                z,
                start: f[0],
                end: f[1],
            });
        }

        if f[0] < f_min || f[0] > f_max {
            return Err(Error::FOutOfRange { f: f[0], z });
        }
        if f[1] < f_min || f[1] > f_max {
            return Err(Error::FOutOfRange { f: f[1], z });
        }

        if x[0] > xy_max {
            return Err(Error::XOutOfRange { x: x[0], z });
        }
        if x[1] > xy_max {
            return Err(Error::XOutOfRange { x: x[1], z });
        }

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
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;
        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i64.pow(difference as u32);
        let scale_xy = 2_u64.pow(difference as u32);

        let f = helpers::scale_range_i64(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u64(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u64(self.y[0], self.y[1], scale_xy);

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
impl SpaceID for RangeID {
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }

    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }

    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    /* -----------------------------
     *     bound_*  (非循環、境界で Err)
     * ----------------------------- */

    fn bound_up(&mut self, by: i64) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        let z = self.z;

        let ns = self.f[0]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i64::MAX, z })?;

        if ns < min || ns > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        if ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ne, z });
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn bound_down(&mut self, by: i64) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        let z = self.z;

        let ns = self.f[0]
            .checked_sub(by)
            .ok_or(Error::FOutOfRange { f: i64::MIN, z })?;
        let ne = self.f[1]
            .checked_sub(by)
            .ok_or(Error::FOutOfRange { f: i64::MIN, z })?;

        if ns < min || ns > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        if ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ne, z });
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn bound_north(&mut self, by: u64) -> Result<(), Error> {
        let max = self.max_xy();
        let z = self.z;

        let ns = self.y[0]
            .checked_add(by)
            .ok_or(Error::YOutOfRange { y: u64::MAX, z })?;
        let ne = self.y[1]
            .checked_add(by)
            .ok_or(Error::YOutOfRange { y: u64::MAX, z })?;

        if ns > max {
            return Err(Error::YOutOfRange { y: ns, z });
        }
        if ne > max {
            return Err(Error::YOutOfRange { y: ne, z });
        }

        self.y = [ns, ne];
        Ok(())
    }

    fn bound_south(&mut self, by: u64) -> Result<(), Error> {
        let max = self.max_xy();
        let z = self.z;

        let ns = self.y[0]
            .checked_sub(by)
            .ok_or(Error::YOutOfRange { y: 0, z })?;
        let ne = self.y[1]
            .checked_sub(by)
            .ok_or(Error::YOutOfRange { y: 0, z })?;

        if ns > max {
            return Err(Error::YOutOfRange { y: ns, z });
        }
        if ne > max {
            return Err(Error::YOutOfRange { y: ne, z });
        }

        self.y = [ns, ne];
        Ok(())
    }

    fn bound_east(&mut self, by: u64) -> Result<(), Error> {
        let max = self.max_xy();
        let z = self.z;

        let ns = self.x[0]
            .checked_add(by)
            .ok_or(Error::XOutOfRange { x: u64::MAX, z })?;
        let ne = self.x[1]
            .checked_add(by)
            .ok_or(Error::XOutOfRange { x: u64::MAX, z })?;

        if ns > max {
            return Err(Error::XOutOfRange { x: ns, z });
        }
        if ne > max {
            return Err(Error::XOutOfRange { x: ne, z });
        }

        self.x = [ns, ne];
        Ok(())
    }

    fn bound_west(&mut self, by: u64) -> Result<(), Error> {
        let max = self.max_xy();
        let z = self.z;

        let ns = self.x[0]
            .checked_sub(by)
            .ok_or(Error::XOutOfRange { x: 0, z })?;
        let ne = self.x[1]
            .checked_sub(by)
            .ok_or(Error::XOutOfRange { x: 0, z })?;

        if ns > max {
            return Err(Error::XOutOfRange { x: ns, z });
        }
        if ne > max {
            return Err(Error::XOutOfRange { x: ne, z });
        }

        self.x = [ns, ne];
        Ok(())
    }

    /* -----------------------------
     *     wrap_*   (循環、エラーなし)
     * ----------------------------- */

    fn wrap_up(&mut self, by: i64) {
        let min = self.min_f();
        let max = self.max_f();
        let width = (max - min + 1) as i128;

        for v in &mut self.f {
            let offset = (*v - min) as i128;
            let new = ((offset + by as i128) % width + width) % width;
            *v = (min as i128 + new) as i64;
        }
    }

    fn wrap_down(&mut self, by: i64) {
        self.wrap_up(-by);
    }

    fn wrap_north(&mut self, by: u64) {
        let max = self.max_xy();
        let ring = max + 1;

        for v in &mut self.y {
            *v = ((*v + by) % ring);
        }
    }

    fn wrap_south(&mut self, by: u64) {
        let max = self.max_xy();
        let ring = max + 1;

        for v in &mut self.y {
            *v = ((*v + ring - (by % ring)) % ring);
        }
    }

    fn wrap_east(&mut self, by: u64) {
        let max = self.max_xy();
        let ring = max + 1;

        for v in &mut self.x {
            *v = ((*v + by) % ring);
        }
    }

    fn wrap_west(&mut self, by: u64) {
        let max = self.max_xy();
        let ring = max + 1;

        for v in &mut self.x {
            *v = ((*v + ring - (by % ring)) % ring);
        }
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
