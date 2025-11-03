use crate::r#type::bit_vec::BitVec;

pub fn invert_bitmask_xy(bitmask: &BitVec) -> (u64, u8) {
    let bytes = &bitmask.0;
    let total_bits = bytes.len() * 8;
    let total_layers = (total_bits + 1) / 2;

    let mut uxy: u64 = 0;
    let mut max_z: i32 = -1; // 見つかった最大のz

    // now_z=0 から順に処理
    for now_z in 0..total_layers {
        let index = (now_z * 2) / 8;
        let in_index = now_z % 4;

        let byte = bytes[index];
        let valid = (byte >> (7 - in_index * 2)) & 1;
        let branch = (byte >> (6 - in_index * 2)) & 1;

        if valid == 1 {
            max_z = now_z as i32;
            // now_z の位置に branch を配置
            uxy |= (branch as u64) << now_z;
        }
    }

    // uxy を反転（ビットの並びを逆にする）
    let final_z = max_z as u8;
    let mut reversed_uxy = 0u64;
    for i in 0..=final_z {
        let bit = (uxy >> i) & 1;
        reversed_uxy |= bit << (final_z - i);
    }

    (reversed_uxy, final_z)
}
