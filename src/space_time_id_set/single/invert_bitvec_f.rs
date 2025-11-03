use crate::r#type::bit_vec::BitVec;

pub fn invert_bitmask_f(bitmask: &BitVec) -> (i64, u8) {
    let bytes = &bitmask.0;
    let total_bits = bytes.len() * 8;
    let total_layers = (total_bits + 1) / 2;

    let mut f: i64 = 0;
    let mut max_z: i32 = -1;
    let mut is_negative = false;

    for now_z in 0..total_layers {
        let index = (now_z * 2) / 8;
        let in_index = now_z % 4;

        let byte = bytes[index];
        let valid = (byte >> (7 - in_index * 2)) & 1;
        let branch = (byte >> (6 - in_index * 2)) & 1;

        if valid == 1 {
            max_z = now_z as i32;

            if now_z == 0 {
                // 最初の階層は符号情報
                is_negative = branch == 1;
            } else {
                // それ以降は各ビットを復元
                // 下位ビットから順に格納されているので、適切な位置に配置
                f |= (branch as i64) << (now_z - 1);
            }
        }
    }

    // 符号を適用
    if is_negative {
        f = -f;
    }

    (f, max_z as u8)
}
