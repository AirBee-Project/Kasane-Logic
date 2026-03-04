use crate::{Segment, spatial_id::flex_id::segment::Bit};

impl<const N: usize> Segment<N> {
    ///セグメントをズームレベルとインデックス値に変換する。
    pub(crate) fn to_xy(&self) -> (u8, u32) {
        let mut z: u8 = 0;
        let mut index = 0;

        'outer: for byte in self.0 {
            for bit_index in 0..=3 {
                let shift = (3 - bit_index) * 2;
                let masked = (byte >> shift) & 0b11;

                match masked {
                    0b10 => {
                        index = index * 2;
                        z += 1;
                    }
                    0b11 => {
                        index = index * 2 + 1;
                        z += 1;
                    }
                    _ => break 'outer,
                }
            }
        }
        let final_z = if z > 0 { z - 1 } else { 0 };
        (final_z, index)
    }

    ///セグメントをズームレベルとインデックス値に変換する。
    pub(crate) fn to_f(&self) -> (u8, i32) {
        let is_negative = self.top_bit_pair() == Bit::One;
        let mut temp = self.clone();
        temp.clear_bit_pair(0);
        temp.set_bit_pair(0, Bit::Zero);

        let (z, u_dim) = temp.to_xy();

        let dim = if is_negative {
            -((u_dim as i32) + 1)
        } else {
            u_dim as i32
        };
        (z, dim)
    }

    ///XYのズームレベルとRangeから最適配置のセグメントを作成。
    pub(crate) fn split_xy(z: u8, range: [u32; 2]) -> impl Iterator<Item = Segment<N>> {
        let [l, r] = range;
        SegmentIter {
            l: l as i32,
            r: r as i32,
            cur_z: z as i8,
        }
        .map(|(z, dim)| Segment::from_xy(z, dim as u32))
    }

    ///FのズームレベルとRangeから最適配置のセグメントを作成。
    pub(crate) fn split_f(z: u8, range: [i32; 2]) -> impl Iterator<Item = Segment<N>> {
        let diff = 1i32 << z;
        let [l, r] = range;
        SegmentIter {
            l: l + diff,
            r: r + diff,
            cur_z: z as i8,
        }
        .map(move |(seg_z, dim)| {
            let original_dim = dim - (1i32 << seg_z);
            Segment::from_f(seg_z, original_dim)
        })
    }

    ///XYのズームレベルと値からセグメントを作成。
    pub(crate) fn from_xy(z: u8, mut dimension: u32) -> Self {
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

    ///Fのズームレベルと値からセグメントを作成。
    pub(crate) fn from_f(z: u8, dimension: i32) -> Self {
        let is_negative = dimension.is_negative();
        let u_dim = if is_negative {
            (dimension.abs() - 1) as u32
        } else {
            dimension as u32
        };

        let mut segment = Self::from_xy(z, u_dim);

        // F次元特有：Z=0 (符号ビット) を上書き
        segment.clear_bit_pair(0);
        segment.set_bit_pair(0, if is_negative { Bit::One } else { Bit::Zero });

        segment
    }
}

struct SegmentIter {
    l: i32,
    r: i32,
    cur_z: i8,
}

impl Iterator for SegmentIter {
    type Item = (u8, i32); // (z, dimension)

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
            if self.l == self.r {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.l & 1 == 1 {
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
