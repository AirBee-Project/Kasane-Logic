use crate::spatial_id::segment::encode::{Bit, EncodeSegment};

pub mod encode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment<T> {
    z: u8,
    dimension: T,
}

impl Segment<u32> {
    /// XY 向けセグメント分割
    pub fn new(z: u8, dimension: [u32; 2]) -> impl Iterator<Item = Segment<u32>> {
        let [l, r] = dimension;

        SegmentIter {
            l: l as i32,
            r: r as i32,
            cur_z: z as i8,
        }
        .map(|seg| Segment {
            z: seg.z,
            dimension: seg.dimension as u32,
        })
    }

    pub fn as_z(&self) -> u8 {
        self.z
    }

    pub fn as_dimension(&self) -> u32 {
        self.dimension
    }
}


impl Segment<i32> {
    /// F 向けセグメント分割
    pub fn new(z: u8, dimension: [i32; 2]) -> impl Iterator<Item = Segment<i32>> {
        let diff = 1i32 << z;
        let [l, r] = dimension;

        SegmentIter {
            l: l + diff,
            r: r + diff,
            cur_z: z as i8,
        }
        .map(move |seg| Segment {
            z: seg.z,
            dimension: seg.dimension - (1i32 << seg.z),
        })
    }

    pub fn as_z(&self) -> u8 {
        self.z
    }

    pub fn as_dimension(&self) -> i32 {
        self.dimension
    }
}

struct SegmentIter {
    l: i32,
    r: i32,
    cur_z: i8,
}

impl Iterator for SegmentIter {
    type Item = Segment<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // 区間が空なら終了
            if self.l > self.r {
                return None;
            }

            // z=0 は最終解像度：そのまま全要素を吐く
            if self.cur_z == 0 {
                let v = self.l;
                self.l += 1;
                return Some(Segment {
                    z: 0,
                    dimension: v,
                });
            }

            let z = self.cur_z as u8;

            // 単一点
            if self.l == self.r {
                let v = self.l;
                self.l += 1;
                return Some(Segment { z, dimension: v });
            }

            // 左端が奇数 → まとめ不可
            if self.l & 1 == 1 {
                let v = self.l;
                self.l += 1;
                return Some(Segment { z, dimension: v });
            }

            // 右端が偶数 → まとめ不可
            if self.r & 1 == 0 {
                let v = self.r;
                self.r -= 1;
                return Some(Segment { z, dimension: v });
            }

            // 親レベルへ昇る
            self.l >>= 1;
            self.r >>= 1;
            self.cur_z -= 1;
        }
    }
}

impl From<EncodeSegment> for Segment<u32> {
    fn from(encode: EncodeSegment) -> Self {
        let mut z: u8 = 0;
        let mut index = 0;

        'outer: for byte in encode.0 {
            for bit_index in 0..=3 {
                let masked =
                    ((0b11000000 >> bit_index * 2) & byte) << bit_index * 2;

                if masked == 0b10000000 {
                    index = index * 2;
                    z += 1;
                } else if masked == 0b11000000 {
                    index = index * 2 + 1;
                    z += 1;
                } else {
                    break 'outer;
                }
            }
        }

        Segment {
            z: z - 1, // 初期 z=0 分を相殺
            dimension: index,
        }
    }
}

impl From<EncodeSegment> for Segment<i32> {
    fn from(mut encode: EncodeSegment) -> Self {
        // z=0 の bit pair は符号
        let is_negative = encode.top_bit_pair() == Bit::One;

        // 符号を除去して u32 として復元
        encode.clear_bit_pair(0);
        encode.set_bit_pair(0, Bit::Zero);

        let u_seg = Segment::<u32>::from(encode);

        let dimension = if is_negative {
            -((u_seg.dimension as i32) + 1)
        } else {
            u_seg.dimension as i32
        };

        Segment {
            z: u_seg.z,
            dimension,
        }
    }
}
