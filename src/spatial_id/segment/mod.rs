use crate::spatial_id::{constants::MAX_ZOOM_LEVEL, segment::encode::EncodeSegment};

pub mod encode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment<T> {
    z: u8,
    dimension: T,
}

impl Segment<u64> {
    /// XY向けのセグメント分割
    pub fn new(z: u8, dimension: [u64; 2]) -> impl Iterator<Item = Segment<u64>> {
        let [l, r] = dimension;

        SegmentIter {
            z,
            l: l as i64,
            r: r as i64,
            cur_z: z,
        }
        .map(|seg| Segment {
            z: seg.z,
            dimension: seg.dimension as u64,
        })
    }

    pub fn as_z(&self) -> u8 {
        self.z
    }

    pub fn as_dimension(&self) -> u64 {
        self.dimension
    }
}

impl Segment<i64> {
    /// F向けのセグメント分割
    pub fn new(z: u8, dimension: [i64; 2]) -> impl Iterator<Item = Segment<i64>> {
        let diff = 1i64 << z;
        let [l, r] = dimension;

        SegmentIter {
            z,
            l: l + diff,
            r: r + diff,
            cur_z: z,
        }
        .map(move |seg| Segment {
            z: seg.z,
            dimension: seg.dimension - (1i64 << seg.z),
        })
    }
    pub fn as_z(&self) -> u8 {
        self.z
    }

    pub fn as_dimension(&self) -> i64 {
        self.dimension
    }
}

struct SegmentIter {
    z: u8,
    l: i64,
    r: i64,
    cur_z: u8,
}

impl Iterator for SegmentIter {
    type Item = Segment<i64>;

    fn next(&mut self) -> Option<Self::Item> {
        // 区間が空になったら終了
        if self.l > self.r {
            return None;
        }

        let z = self.cur_z;

        // 単一点
        if self.l == self.r {
            let v = self.l;
            self.l += 1;
            return Some(Segment { z, dimension: v });
        }

        // 左端が奇数 → 親セルにまとめられない
        if self.l & 1 == 1 {
            let v = self.l;
            self.l += 1;
            return Some(Segment { z, dimension: v });
        }

        // 右端が偶数 → 親セルにまとめられない
        if self.r & 1 == 0 {
            let v = self.r;
            self.r -= 1;
            return Some(Segment { z, dimension: v });
        }

        // これ以上解像度を下げられない
        if self.cur_z == 0 {
            return None;
        }

        // 親レベルへ昇る
        self.l >>= 1;
        self.r >>= 1;
        self.cur_z -= 1;

        self.next()
    }
}

impl From<EncodeSegment> for Segment<u64> {
    fn from(encode: EncodeSegment) -> Self {
        let mut z: u8 = 0;
        let mut index = 0;

        'outer: for byte in encode.0 {
            for bit_index in 0..=3 {
                let masked: u8 = ((0b11000000 >> bit_index * 2) & byte) << bit_index * 2;

                if masked == 0b10000000 {
                    index = index * 2;
                    z = z + 1;
                } else if masked == 0b11000000 {
                    index = index * 2 + 1;
                    z = z + 1;
                } else {
                    break 'outer;
                }
            }
        }

        Segment {
            //初期のZ=0の処理を相殺
            z: z - 1,
            dimension: index,
        }
    }
}

impl From<EncodeSegment> for Segment<i64> {
    fn from(encode: EncodeSegment) -> Self {
        let mut z: u8 = 0;
        let mut index = -1;

        'outer: for byte in encode.0 {
            for bit_index in 0..=3 {
                let masked: u8 = ((0b11000000 >> bit_index * 2) & byte) << bit_index * 2;

                if masked == 0b10000000 {
                    index = index * 2;
                    z = z + 1;
                } else if masked == 0b11000000 {
                    index = index * 2 + 1;
                    z = z + 1;
                } else {
                    break 'outer;
                }
            }
        }

        Segment {
            z: z,
            dimension: index,
        }
    }
}
