use std::num::{NonZero, NonZeroU64};

use crate::{Segment, TemporalId};

impl TemporalId {
    pub fn split_segments(&self) -> impl Iterator<Item = Segment<16>> {
        let l = (self.t[0] as i128) * (self.i.get() as i128);
        let r = (self.t[1] as i128) * (self.i.get() as i128) + (self.i.get() as i128) - 1;

        TemporalSegmentIter { l, r, cur_z: 64 }.map(|(z, dim)| Segment::from_t(z, dim as u64))
    }
}

impl From<Segment<16>> for TemporalId {
    fn from(segment: Segment<16>) -> Self {
        let (z, index) = segment.to_t();
        let shift = 64 - z;

        let start_idx = ((index as u128) << shift) as u64;
        let end_idx = (((index as u128 + 1) << shift) - 1) as u64;

        Self {
            i: NonZeroU64::new(1).unwrap(),
            t: [start_idx, end_idx],
        }
    }
}

struct TemporalSegmentIter {
    l: i128,
    r: i128,
    cur_z: i8,
}

impl Iterator for TemporalSegmentIter {
    type Item = (u8, i128);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.l > self.r {
                return None;
            }

            if self.cur_z == 0 || self.l == self.r || self.l % 2 == 1 {
                let z = self.cur_z as u8;
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }

            if self.r % 2 == 0 {
                let z = self.cur_z as u8;
                let v = self.r;
                self.r -= 1;
                return Some((z, v));
            }

            self.l >>= 1;
            self.r >>= 1;
            self.cur_z -= 1;
        }
    }
}
