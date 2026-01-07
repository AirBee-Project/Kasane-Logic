use std::fmt;
use std::fmt::Display;

use crate::segment::Segment;
use crate::spatial_id::constants::MAX_ZOOM_LEVEL;

#[derive(Debug, Clone, PartialEq)]
pub struct EncodeSegment([u8; EncodeSegment::ARRAY_LENGTH]);

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
    const ARRAY_LENGTH: usize = (MAX_ZOOM_LEVEL * 2).div_ceil(8);
}

impl From<Segment<u64>> for EncodeSegment {
    fn from(segment: Segment<u64>) -> Self {
        let mut result = [0u8; EncodeSegment::ARRAY_LENGTH];
        let mut index_num = segment.as_dimension();

        for z in 0..segment.as_z() {
            //ZoomLeveL=0では無条件で0に設定
            if z == 0 {
                set_bit_pair(&mut result, 0, Bit::Zero);
                continue;
            }

            if index_num % 2 == 0 {
                set_bit_pair(&mut result, z, Bit::Zero);
            } else {
                set_bit_pair(&mut result, z, Bit::One);
            }

            index_num = index_num / 2;
        }
        Self(result)
    }
}

impl From<Segment<i64>> for EncodeSegment {
    fn from(segment: Segment<i64>) -> Self {
        let mut result = [0u8; EncodeSegment::ARRAY_LENGTH];
        let mut index_num = segment.as_dimension();
        for z in 0..segment.as_z() {
            if index_num % 2 == 0 {
                set_bit_pair(&mut result, z, Bit::Zero);
            } else {
                set_bit_pair(&mut result, z, Bit::One);
            }

            index_num = index_num / 2;
        }
        Self(result)
    }
}

///ある階層の情報をセットする
/// 上下階層との整合性などは保証せず、呼び出し側が保証を行う
/// 対象のBitが`00`であることが呼び出し条件
fn set_bit_pair(encode_segment: &mut [u8; EncodeSegment::ARRAY_LENGTH], z: u8, bit: Bit) {
    let byte_index = (z / 4) as usize;
    let bit_index = (z % 4) * 2;

    let byte = &mut encode_segment[byte_index];

    // 対象 2bit をクリア
    // let mask = !(0b11 << bit_index);
    // *byte &= mask;

    // 新しい 2bit を作成
    let new_bits = match bit {
        Bit::Zero => 0b10000000,
        Bit::One => 0b11000000,
    } >> bit_index;

    // 設定
    *byte |= new_bits;
}
