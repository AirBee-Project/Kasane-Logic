use crate::{Segment, TemporalId};

impl TemporalId {
    pub fn split_segments<const N: usize>(&self) -> impl Iterator<Item = Segment<N>> {
        let [l, r] = self.t;
        TemporalSegmentIter {
            l: l as i128,
            r: r as i128,
            cur_z: 64,
        }
        .map(|(z, dim)| Segment::from_t(z, dim as u64))
    }
}

impl<const N: usize> From<Segment<N>> for TemporalId {
    fn from(segment: Segment<N>) -> Self {
        let (z, index) = segment.to_t();
        let shift = 64 - z;
        let start_idx = index << shift;
        let end_idx = (((index as u128 + 1) << shift) - 1) as u64;
        Self {
            i: 1, // 1秒単位の解像度
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
