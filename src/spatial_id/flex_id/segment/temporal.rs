use crate::{Segment, spatial_id::flex_id::segment::Bit};

impl<const N: usize> Segment<N> {
    pub(crate) fn from_t(z: u8, mut dimension: u64) -> Self {
        let mut segment = Segment::new([0u8; N]);

        // Z=0 (ルート) のビットをセット
        segment.set_bit_pair(0, Bit::Zero);

        // z=0 の場合は、ルートセグメント（全範囲）を意味する
        if z == 0 {
            return segment;
        }

        // それ以外の場合は指定されたズームレベルまでビットを埋める
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
