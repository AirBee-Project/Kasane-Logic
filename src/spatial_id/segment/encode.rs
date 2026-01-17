use std::fmt;
use std::fmt::Display;

use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::segment::Segment;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncodeSegment(pub(crate) [u8; EncodeSegment::ARRAY_LENGTH]);

impl Display for EncodeSegment {
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

///Bit操作を行う際に安全に`0`と`1`を指定する
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Bit {
    Zero = 0,
    One = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentRelation {
    Equal,
    Ancestor,
    Descendant,
    Disjoint,
}

impl EncodeSegment {
    ///EncodeSegmentの配列長を[MAX_ZOOM_LEVEL]から定義
    pub(crate) const ARRAY_LENGTH: usize = (MAX_ZOOM_LEVEL * 2).div_ceil(8);

    ///ある階層の情報をセットする
    /// 上下階層との整合性などは保証せず、呼び出し側が保証を行う
    /// 対象のBitが`00`であることが呼び出し条件
    pub(super) fn set_bit_pair(&mut self, z: u8, bit: Bit) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;

        let byte = &mut self.0[byte_index];

        // 新しい 2bit を作成
        let new_bits: u8 = match bit {
            Bit::Zero => 0b10000000,
            Bit::One => 0b11000000,
        } >> bit_index;

        // 設定
        *byte |= new_bits;
    }

    ///ある階層の情報を`00`にリセットする
    pub(super) fn clear_bit_pair(&mut self, z: u8) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        let byte = &mut self.0[byte_index];
        let mask = !(0b11000000 >> bit_index);
        *byte &= mask;
    }

    pub(super) fn top_bit_pair(&self) -> Bit {
        let pair = self.0[0] & 0b11000000;
        match pair {
            0b10000000 => Bit::Zero,
            0b11000000 => Bit::One,
            _ => unreachable!("bit pair at z=0 is not set"),
        }
    }

    ///関係を返す関数
    pub fn relation(&self, other: &EncodeSegment) -> SegmentRelation {
        for (b1, b2) in self.0.iter().zip(other.0.iter()) {
            // バイトが完全に一致していれば、そのバイト内の全階層は同じ道を辿っている
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

                // NOTE: 0 = 0b00 (無効/パディング), 非0 = 0b10 or 0b11 (有効)
                return match (p1, p2) {
                    (0, _) => SegmentRelation::Ancestor, // 自分は終わってるが相手は続いてる -> 上位
                    (_, 0) => SegmentRelation::Descendant, // 相手は終わってるが自分は続いてる -> 下位
                    (_, _) => SegmentRelation::Disjoint,   // 両方有効だが値が違う -> 排反
                };
            }
        }

        // 最後まで違いが見つからなかった場合
        SegmentRelation::Equal
    }

    ///兄弟セグメントを返す関数
    pub fn sibling(&self) -> EncodeSegment {
        let mut out = self.clone();

        // 下位レベルから探索
        for byte_index in (0..out.0.len()).rev() {
            let byte = out.0[byte_index];

            // このバイトに有効 bit-pair が無ければスキップ
            if byte == 0 {
                continue;
            }

            // 1バイト内を下位ペアから確認
            for shift in [0, 2, 4, 6] {
                let mask = 0b11 << shift;
                let pair = (byte & mask) >> shift;

                if pair == 0 {
                    continue;
                }

                // 10 <-> 11 を反転
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

        unreachable!("このセグメントは無効です");
    }

    /// 1つ上の階層（親セグメント）を返す
    pub fn parent(&self) -> Option<EncodeSegment> {
        let mut out = self.clone();

        // 下位から探索
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

                // 最下位の有効 bit-pair を消す
                out.0[byte_index] &= !mask;
                return Some(out);
            }
        }

        // すでに root（これ以上上がれない）
        None
    }

    ///下位のセグメントを検索するための範囲の下限を返す関数
    pub fn children_range_end(&self) -> EncodeSegment {
        let mut end_segment = self.clone();
        let max_z = (Self::ARRAY_LENGTH * 4) as u8 - 1;

        for z in (0..=max_z).rev() {
            let pair = end_segment.get_raw_pair(z);

            match pair {
                0b00 => continue,
                0b10 => {
                    end_segment.set_bit_pair(z, Bit::One);
                    return end_segment;
                }
                0b11 => {
                    end_segment.clear_bit_pair(z);
                }
                _ => unreachable!("Invalid bit pair detected"),
            }
        }
        end_segment
    }

    fn get_raw_pair(&self, z: u8) -> u8 {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;
        // マスクして取り出し、右端(LSB)に寄せる
        (self.0[byte_index] >> (6 - bit_index)) & 0b11
    }
}

impl From<Segment<u32>> for EncodeSegment {
    fn from(segment: Segment<u32>) -> Self {
        let mut result = EncodeSegment([0u8; EncodeSegment::ARRAY_LENGTH]);
        let mut index_num = segment.as_dimension();
        for z in (0..=segment.as_z()).rev() {
            //ZoomLeveL=0では無条件で0に設定
            if z == 0 {
                result.set_bit_pair(0, Bit::Zero);
                continue;
            }

            if index_num % 2 == 0 {
                result.set_bit_pair(z, Bit::Zero);
            } else {
                result.set_bit_pair(z, Bit::One);
            }

            index_num = index_num / 2;
        }
        result
    }
}

impl From<Segment<i32>> for EncodeSegment {
    fn from(segment: Segment<i32>) -> Self {
        let is_negative = segment.dimension.is_negative();

        // Segment<u64>に変換して考える
        let u64_dimension = if is_negative {
            (segment.dimension.abs() - 1) as u32
        } else {
            segment.dimension as u32
        };

        let u64_segment = Segment {
            z: segment.z,
            dimension: u64_dimension,
        };

        let mut encoded = EncodeSegment::from(u64_segment);

        // ZoomLeveL=0の分岐のみを上書き
        encoded.clear_bit_pair(0);
        encoded.set_bit_pair(0, if is_negative { Bit::One } else { Bit::Zero });

        encoded
    }
}
