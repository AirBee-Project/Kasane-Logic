pub mod impls;
pub mod spatial;
pub mod temporal;

///Segmentを`V-Bit`を用いて表す。
/// Nにはこのセグメントで使用したいByte数を入力
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment<const N: usize>([u8; N]);

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
    ///等価なセグメント。
    Equal,

    ///上位セグメント。
    Ancestor,

    ///下位セグメント。
    Descendant,

    ///関連のないセグメント。
    Disjoint,
}

impl<const N: usize> Segment<N> {
    pub const ARRAY_LENGTH: usize = N;

    pub fn new(bytes: [u8; N]) -> Self {
        Segment(bytes)
    }

    ///セグメント同士の関連を把握する。
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

    ///兄弟セグメントを求める。
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
        unreachable!("Neko")
    }

    ///そのセグメントの1階層上位のセグメントを返す
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
                if byte_index == 0 && shift == 6 {
                    return None;
                }
                out.0[byte_index] &= !mask;
                return Some(out);
            }
        }
        None
    }

    ///自分を含んで、順番に親を返していく
    pub fn self_and_parents(&self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self.clone()), |node| node.parent())
    }

    ///下位セグメントを検索する場合の検索範囲の右点を返す。
    pub fn descendant_range_end(&self) -> Option<Self> {
        let mut end_segment = self.clone();
        let max_z = (N * 4) as u8 - 1;

        for z in (0..=max_z).rev() {
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

    ///セグメントからセグメントを引く。
    pub(crate) fn difference(&self, other: &Self) -> Vec<Segment<N>> {
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

    ///ある階層のBitPairを設定する。
    fn set_bit_pair(&mut self, z: u8, bit: Bit) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let new_bits: u8 = match bit {
            Bit::Zero => 0b10000000,
            Bit::One => 0b11000000,
        } >> bit_index;
        self.0[byte_index] |= new_bits;
    }

    ///ある階層のBitPairを`00`に置換する。
    fn clear_bit_pair(&mut self, z: u8) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let mask = !(0b11000000 >> bit_index);
        self.0[byte_index] &= mask;
    }

    ///一番上位の階層の分割Bitが`0`か`1`かを返す。
    fn top_bit_pair(&self) -> Bit {
        match self.0[0] & 0b11000000 {
            0b10000000 => Bit::Zero,
            0b11000000 => Bit::One,
            _ => unreachable!("bit pair at z=0 is not set"),
        }
    }
}
