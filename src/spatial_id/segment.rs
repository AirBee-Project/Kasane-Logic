use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment([u8; Segment::ARRAY_LENGTH]);

/// 内部ビット表現のヘルパー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Bit {
    Zero = 0,
    One = 1,
}

/// 2つのセグメントの位置関係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentRelation {
    Equal,
    Ancestor,
    Descendant,
    Disjoint,
}

impl Segment {
    pub(crate) const ARRAY_LENGTH: usize = (MAX_ZOOM_LEVEL * 2).div_ceil(8);

    pub(crate) fn split_xy(z: u8, range: [u32; 2]) -> impl Iterator<Item = Segment> {
        let [l, r] = range;
        SegmentIter {
            l: l as i32,
            r: r as i32,
            cur_z: z as i8,
        }
        .map(|(z, dim)| Segment::from_xy(z, dim as u32))
    }

    pub(crate) fn split_f(z: u8, range: [i32; 2]) -> impl Iterator<Item = Segment> {
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

    pub(crate) fn from_xy(z: u8, mut dimension: u32) -> Self {
        let mut segment = Segment([0u8; Self::ARRAY_LENGTH]);

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

    pub(crate) fn to_xy(&self) -> (u8, u32) {
        let mut z: u8 = 0;
        let mut index = 0;

        'outer: for byte in self.0 {
            for bit_index in 0..=3 {
                let shift = (3 - bit_index) * 2;
                let masked = (byte >> shift) & 0b11;

                match masked {
                    0b10 => {
                        // Zero (0)
                        index = index * 2;
                        z += 1;
                    }
                    0b11 => {
                        // One (1)
                        index = index * 2 + 1;
                        z += 1;
                    }
                    _ => break 'outer, // 00 (End)
                }
            }
        }
        // z=0 (ルート) 分を引く
        let final_z = if z > 0 { z - 1 } else { 0 };
        (final_z, index)
    }

    /// 情報を復元する (F次元用) -> (z, dimension)
    pub(crate) fn to_f(&self) -> (u8, i32) {
        let is_negative = self.top_bit_pair() == Bit::One;

        // 符号ビットを除外して計算するためにクローンして上書き
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

    pub(crate) fn relation(&self, other: &Self) -> SegmentRelation {
        for (b1, b2) in self.0.iter().zip(other.0.iter()) {
            if b1 == b2 {
                continue;
            }

            for shift in [6, 4, 2, 0] {
                let mask = 0b11 << shift;
                let p1 = (b1 & mask) >> shift;
                let p2 = (b2 & mask) >> shift;

                if p1 == p2 {
                    continue;
                }

                return match (p1, p2) {
                    (0, _) => SegmentRelation::Ancestor,
                    (_, 0) => SegmentRelation::Descendant,
                    (_, _) => SegmentRelation::Disjoint,
                };
            }
        }
        SegmentRelation::Equal
    }

    pub(crate) fn sibling(&self) -> Self {
        let mut out = self.clone();
        for byte_index in (0..out.0.len()).rev() {
            let byte = out.0[byte_index];
            if byte == 0 {
                continue;
            }

            for shift in [0, 2, 4, 6] {
                let mask = 0b11 << shift;
                let pair = (byte & mask) >> shift;
                if pair == 0 {
                    continue;
                }

                // 10 <-> 11 flip
                let flipped = match pair {
                    0b10 => 0b11,
                    0b11 => 0b10,
                    _ => unreachable!(),
                } << shift;

                out.0[byte_index] &= !mask;
                out.0[byte_index] |= flipped;
                return out;
            }
        }
        println!("Neko");
        unreachable!("Neko")
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        let mut out = self.clone();
        for byte_index in (0..out.0.len()).rev() {
            let byte = out.0[byte_index];
            if byte == 0 {
                continue;
            }

            for shift in [0, 2, 4, 6] {
                let mask = 0b11 << shift;
                let pair = (byte & mask) >> shift;
                if pair == 0 {
                    continue;
                }

                // 最後に見つかった有効ビットペアを00(消去)にする
                out.0[byte_index] &= !mask;
                return Some(out);
            }
        }
        None
    }

    pub(crate) fn descendant_range_end(&self) -> Option<Self> {
        let mut end_segment = self.clone();
        let max_z = (Self::ARRAY_LENGTH * 4) as u8 - 1;

        for z in (0..=max_z).rev() {
            let byte_index = (z / 4) as usize;
            let bit_index = (z % 4) * 2;
            let shift = 6 - bit_index;

            let pair = (end_segment.0[byte_index] >> shift) & 0b11;

            match pair {
                0b00 => continue,
                0b10 => {
                    // 0 -> 1 にして返す
                    end_segment.set_bit_pair(z, Bit::One);
                    return Some(end_segment);
                }
                0b11 => {
                    // 1 -> 0 にして上位へ
                    end_segment.clear_bit_pair(z);
                }
                _ => unreachable!(),
            }
        }
        None
    }

    pub(crate) fn difference(&self, other: &Self) -> Vec<Segment> {
        if self == other {
            return vec![];
        }
        let mut results = Vec::new();
        let mut current = other.clone();
        while &current != self {
            results.push(current.sibling());
            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
        }
        results
    }

    fn set_bit_pair(&mut self, z: u8, bit: Bit) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let new_bits: u8 = match bit {
            Bit::Zero => 0b10000000,
            Bit::One => 0b11000000,
        } >> bit_index;
        self.0[byte_index] |= new_bits;
    }

    fn clear_bit_pair(&mut self, z: u8) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let mask = !(0b11000000 >> bit_index);
        self.0[byte_index] &= mask;
    }

    fn top_bit_pair(&self) -> Bit {
        match self.0[0] & 0b11000000 {
            0b10000000 => Bit::Zero,
            0b11000000 => Bit::One,
            _ => unreachable!("bit pair at z=0 is not set"),
        }
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

impl Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:08b}", byte)?;
        }
        Ok(())
    }
}
