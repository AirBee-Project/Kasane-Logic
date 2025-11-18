use crate::{bit_vec::BitVec, space_time_id_set::single::invert_bitvec_xy::invert_bitmask_xy};

pub fn invert_bitmask_f(bitmask: &BitVec) -> (u8, i64) {
    //負の範囲が考慮されていないので、おかしい

    let (z, f) = invert_bitmask_xy(bitmask);

    println!("BIT{}", bitmask);
    println!("F  {}", f);
    println!("Z  {}", z);

    //仮に負の範囲の場合
    if *bitmask.0.first().unwrap() >= 0b11000000 {
        return (z, -(f as i64) + 2_i64.pow(z.into()));
    } else {
        return (z, f as i64);
    }
}
