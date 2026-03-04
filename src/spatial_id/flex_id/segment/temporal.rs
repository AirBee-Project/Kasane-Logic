use crate::{Segment, spatial_id::flex_id::segment::Bit};

impl<const N: usize> Segment<N> {
    pub(crate) fn split_t(z: u8, range: [u64; 2]) -> impl Iterator<Item = Segment<N>> {
        let [l, r] = range;
        TemporalSegmentIter {
            l: l as i128,
            r: r as i128,
            cur_z: z as i8,
        }
        .map(|(z, dim)| Segment::from_t(z, dim as u64))
    }

    /// Tのズームレベル（最大64）と値(u64)からセグメントを作成。
    pub(crate) fn from_t(z: u8, mut dimension: u64) -> Self {
        let mut segment = Segment::new([0u8; N]);

        // Z=0 は常に 0 (ルート)
        segment.set_bit_pair(0, Bit::Zero);

        for cur_z in (1..=z).rev() {
            let bit = if dimension % 2 == 0 {
                Bit::Zero
            } else {
                Bit::One
            };
            segment.set_bit_pair(cur_z, bit);
            dimension /= 2;
        }
        segment
    }

    /// Tのセグメントをズームレベルとインデックス値(u64)に変換する。
    pub(crate) fn to_t(&self) -> (u8, u64) {
        let mut z: u8 = 0;
        let mut index: u64 = 0;

        'outer: for byte in self.0 {
            for bit_index in 0..=3 {
                let shift = (3 - bit_index) * 2;
                let masked = (byte >> shift) & 0b11;

                match masked {
                    0b10 => {
                        index = index << 1;
                        z += 1;
                    }
                    0b11 => {
                        index = (index << 1) | 1;
                        z += 1;
                    }
                    _ => break 'outer,
                }
            }
        }
        let final_z = if z > 0 { z - 1 } else { 0 };
        (final_z, index)
    }
}

struct TemporalSegmentIter {
    l: i128,
    r: i128,
    cur_z: i8,
}

impl Iterator for TemporalSegmentIter {
    type Item = (u8, i128); // (z, dimension)

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.l > self.r {
                return None;
            }

            if self.cur_z == 0 {
                let v = self.l;
                self.l += 1;
                return Some((0, v));
            }

            let z = self.cur_z as u8;
            if self.l == self.r || self.l & 1 == 1 {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.r & 1 == 0 {
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
