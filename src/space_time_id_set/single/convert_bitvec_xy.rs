use crate::bit_vec::BitVec;

pub fn convert_bitmask_xy(z: u8, xy: u64) -> BitVec {
    let length = ((z * 2 / 8) + 1).max(1) as usize;
    let mut result = vec![0u8; length];

    let bit_count = (z + 1) as u32;
    let mask = if bit_count >= 64 {
        u64::MAX
    } else {
        (1u64 << bit_count) - 1
    };
    let uxy = xy & mask;

    for now_z in (0..=z).rev() {
        let index = ((now_z) * 2 / 8) as usize;
        let in_index = now_z % 4;

        // 有効ビット
        result[index] |= 1 << (7 - in_index * 2);

        // MSB側から取得するように変更
        let bit_position = z - now_z; // now_z が大きいときに上位ビットを取る
        if (uxy >> bit_position) & 1 != 0 {
            result[index] |= 1 << (6 - in_index * 2);
        }
    }

    BitVec::from_vec(result)
}
