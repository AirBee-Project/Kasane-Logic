use crate::bit_vec::BitVec;

pub fn invert_bitmask_f(bitmask: &BitVec) -> (u8, i64) {
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

    if is_negative {
        if max_z <= 0 {
            // max_z が 0 の場合も汎用計算
            f = -(1 << 0); // -1
        } else if f == 0 {
            // max_z > 0 で下位ビットが全て 0 の場合
            f = -(1 << (max_z));
        } else {
            f = -f;
        }
    }

    (max_z as u8, f)
}
