use crate::bit_vec::BitVec;

pub fn convert_bitmask_f(z: u8, f: i64) -> BitVec {
    let length = ((z * 2 / 8) + 1) as usize;
    let mut result = vec![0u8; length];

    // 符号を保存するためにfの絶対値を使用
    let is_negative = f < 0;
    let mut f_abs = f.abs();

    // 最初の階層(z=0)の処理 - 符号を格納
    result[0] |= 1 << 7; // 有効ビット
    if is_negative {
        result[0] |= 1 << 6; // 分岐ビット（負の場合）
    }

    // それ以降の階層では各ビットを順に格納
    for now_z in 1..=z {
        let index = ((now_z) * 2 / 8) as usize;
        let in_index = now_z % 4;

        // 有効ビット
        result[index] |= 1 << (7 - in_index * 2);

        // 分岐ビット - f_absの最下位ビットを使用
        if f_abs % 2 != 0 {
            result[index] |= 1 << (6 - in_index * 2);
        }

        f_abs >>= 1; // 次のビットへ
    }

    let result = BitVec::from_vec(result);
    result
}
