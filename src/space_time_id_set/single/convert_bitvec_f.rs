use crate::{bit_vec::BitVec, space_time_id_set::single::convert_bitvec_xy::convert_bitmask_xy};

pub fn convert_bitmask_f(z: u8, f: i64) -> BitVec {
    if f >= 0 {
        //-1しなければならないのはおかしい気がするが、なぜがこれで動いてしまっている
        convert_bitmask_xy(z, f as u64)
    } else {
        let mut converted = convert_bitmask_xy(z, f.unsigned_abs());
        let masked: u8 = 0b11000000;
        converted.0[0] |= masked;

        converted
    }
}
