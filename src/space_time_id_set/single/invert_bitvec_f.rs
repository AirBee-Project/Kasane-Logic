use crate::{bit_vec::HierarchicalKey, space_time_id_set::single::invert_bitvec_xy::invert_bitmask_xy};

pub(crate) fn invert_bitmask_f(bitmask: &HierarchicalKey) -> (u8, i64) {
    //負の範囲が考慮されていないので、おかしい

    let (z, f) = invert_bitmask_xy(bitmask);

    //仮に負の範囲の場合
    if *bitmask.0.first()
        .expect("Internal error: HierarchicalKey is empty in invert_bitmask_f") >= 0b11000000 {
        return (z, -(f as i64) + 2_i64.pow(z.into()));
    } else {
        return (z, f as i64);
    }
}
