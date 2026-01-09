use std::fmt;
use std::fmt::Display;

use crate::spatial_id::constants::MAX_ZOOM_LEVEL;
use crate::spatial_id::segment::Segment;

#[derive(Debug, Clone, PartialEq)]
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

impl EncodeSegment {
    ///EncodeSegmentの配列長を[MAX_ZOOM_LEVEL]から定義
    pub(crate) const ARRAY_LENGTH: usize = (MAX_ZOOM_LEVEL * 2).div_ceil(8);

    ///ある階層の情報をセットする
    /// 上下階層との整合性などは保証せず、呼び出し側が保証を行う
    /// 対象のBitが`00`であることが呼び出し条件
    fn set_bit_pair(&mut self, z: u8, bit: Bit) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;

        let byte = &mut self.0[byte_index];

        // 対象 2bit をクリア
        // let mask = !(0b11 << bit_index);
        // *byte &= mask;

        // 新しい 2bit を作成
        let new_bits: u8 = match bit {
            Bit::Zero => 0b10000000,
            Bit::One => 0b11000000,
        } >> bit_index;

        // 設定
        *byte |= new_bits;
    }

    ///ある階層の情報を`00`にリセットする
    fn clear_bit_pair(&mut self, z: u8) {
        let byte_index = (z / 4) as usize;
        let bit_index = (z % 4) * 2;

        let byte = &mut self.0[byte_index];
        let mask = !(0b11 << bit_index);
        *byte &= mask;
    }
}

impl From<Segment<u64>> for EncodeSegment {
    fn from(segment: Segment<u64>) -> Self {
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

impl From<Segment<i64>> for EncodeSegment {
    fn from(segment: Segment<i64>) -> Self {
        let is_negative = segment.dimension.is_negative();

        // Segment<u64>に変換して考える
        let u64_dimension = if is_negative {
            (segment.dimension.abs() - 1) as u64
        } else {
            segment.dimension as u64
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
