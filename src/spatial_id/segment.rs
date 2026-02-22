use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use std::fmt::{self, Display};

/// Segmentを`V-Bit`を用いて表す。
/// 各次元の階層構造をビットペアで保持し、128Bit〜のサイズで空間の「箱」を表現する。
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
    /// 等価なセグメント。
    Equal,

    /// 上位セグメント。
    Ancestor,

    /// 下位セグメント。
    Descendant,

    /// 関連のないセグメント。
    Disjoint,
}

impl Segment {
    /// ズームレベル 0..=64 を2ビットずつ保持するために必要なバイト長
    pub const ARRAY_LENGTH: usize = ((MAX_ZOOM_LEVEL + 1) * 2).div_ceil(8);

    /// XYのズームレベルとRangeから最適配置のセグメントを作成。
    pub(crate) fn split_xy(z: u8, range: [u64; 2]) -> impl Iterator<Item = Segment> {
        let [l, r] = range;
        SegmentIter {
            l: l as i64,
            r: r as i64,
            cur_z: z as i8,
        }
        .map(move |(seg_z, dim)| Segment::from_xy(seg_z, dim as u64))
    }

    /// FのズームレベルとRangeから最適配置のセグメントを作成。
    pub(crate) fn split_f(z: u8, range: [i64; 2]) -> impl Iterator<Item = Segment> {
        let diff = 1i64 << z;
        let [l, r] = range;
        // 符号を考慮した正規化計算
        SegmentIter {
            l: l.saturating_add(diff),
            r: r.saturating_add(diff),
            cur_z: z as i8,
        }
        .map(move |(seg_z, dim)| {
            let original_dim = dim.saturating_sub(1i64 << seg_z);
            Segment::from_f(seg_z, original_dim)
        })
    }

    /// XYのズームレベルと値からセグメントを作成。
    pub(crate) fn from_xy(z: u8, mut dimension: u64) -> Self {
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

    /// Fのズームレベルと値からセグメントを作成。
    pub(crate) fn from_f(z: u8, dimension: i64) -> Self {
        let is_negative = dimension.is_negative();
        let u_dim = if is_negative {
            (dimension.abs() - 1) as u64
        } else {
            dimension as u64
        };

        let mut segment = Self::from_xy(z, u_dim);

        // F次元特有：Z=0 (符号ビット) を上書き
        segment.clear_bit_pair(0);
        segment.set_bit_pair(0, if is_negative { Bit::One } else { Bit::Zero });

        segment
    }

    /// セグメントをズームレベルとインデックス値に変換する。
    pub(crate) fn to_xy(&self) -> (u8, u64) {
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

    /// セグメントをズームレベルとインデックス値に変換する。
    pub(crate) fn to_f(&self) -> (u8, i64) {
        let is_negative = self.top_bit_pair() == Bit::One;
        let mut temp = self.clone();
        temp.clear_bit_pair(0);
        temp.set_bit_pair(0, Bit::Zero);

        let (z, u_dim) = temp.to_xy();

        let dim = if is_negative {
            -((u_dim as i64) + 1)
        } else {
            u_dim as i64
        };
        (z, dim)
    }

    /// セグメント同士の関連を把握する。
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

    /// 兄弟セグメントを求める。
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
        unreachable!("The root segment has no siblings")
    }

    /// そのセグメントの1階層上位のセグメントを返す
    /// ただし、Z=0に親は存在しないため、その場合はNoneを返す
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
                // Root check (z=0)
                if byte_index == 0 && shift == 6 {
                    return None;
                }
                out.0[byte_index] &= !mask;
                return Some(out);
            }
        }
        None
    }

    /// 自分を含んで、順番に親を返していく
    pub fn self_and_parents(&self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self.clone()), |node| node.parent())
    }

    /// 下位セグメントを検索する場合の検索範囲の右点を返す。
    pub fn descendant_range_end(&self) -> Option<Self> {
        let mut end_segment = self.clone();
        let max_possible_z = (Self::ARRAY_LENGTH * 4) as u8 - 1;
        let limit_z = MAX_ZOOM_LEVEL as u8;

        for z in (0..=limit_z.min(max_possible_z)).rev() {
            let byte_index = (z / 4) as usize;
            let bit_index = (z % 4) * 2;
            let shift = 6 - bit_index;

            let pair = (end_segment.0[byte_index] >> shift) & 0b11;

            match pair {
                0b00 => continue,
                0b10 => {
                    end_segment.set_bit_pair(z, Bit::One);
                    return Some(end_segment);
                }
                0b11 => {
                    end_segment.clear_bit_pair(z);
                }
                _ => unreachable!(),
            }
        }
        None
    }

    /// セグメントからセグメントを引く。
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

    /// ある階層のBitPairを設定する。
    fn set_bit_pair(&mut self, z: u8, bit: Bit) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let new_bits: u8 = match bit {
            Bit::Zero => 0b10000000,
            Bit::One => 0b11000000,
        } >> bit_index;
        self.0[byte_index] |= new_bits;
    }

    /// ある階層のBitPairを`00`に置換する。
    fn clear_bit_pair(&mut self, z: u8) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let mask = !(0b11000000 >> bit_index);
        self.0[byte_index] &= mask;
    }

    /// 一番上位の階層の分割Bitが`0`か`1`かを返す。
    fn top_bit_pair(&self) -> Bit {
        match self.0[0] & 0b11000000 {
            0b10000000 => Bit::Zero,
            0b11000000 => Bit::One,
            _ => unreachable!("bit pair at z=0 is not set"),
        }
    }
}

struct SegmentIter {
    l: i64,
    r: i64,
    cur_z: i8,
}

impl Iterator for SegmentIter {
    type Item = (u8, i64); // (z, dimension)

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

impl From<[u8; Segment::ARRAY_LENGTH]> for Segment {
    fn from(value: [u8; Segment::ARRAY_LENGTH]) -> Self {
        Segment(value)
    }
}

impl From<Segment> for [u8; Segment::ARRAY_LENGTH] {
    fn from(value: Segment) -> Self {
        value.0
    }
}
