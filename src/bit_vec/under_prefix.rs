use crate::bit_vec::BitVec;

impl BitVec {
    /// 下位範囲を検索するときに必要な右側の端点の値を出す
    /// (Start, End) ただし Start < End
    pub fn under_prefix(&self) -> (BitVec, BitVec) {
        (self.clone(), self.next_prefix().clone())
    }
}

impl BitVec {
    /// 自分の右側（prefix の upper bound）を求める
    pub fn next_prefix(&self) -> BitVec {
        let mut out = self.clone();
        let mut levels = out.0.clone();

        // levels を 2bit 単位で操作する
        'outer: for byte_i in 0..levels.len() {
            let byte = levels[byte_i];
            for bitpos in (0..4).rev() {
                // 上位階層から
                let valid = (byte >> (bitpos * 2 + 1)) & 1;
                let dir = (byte >> (bitpos * 2)) & 1;

                if valid == 0 {
                    // ここから下は存在しない
                    break 'outer;
                }

                // dir=0 を dir=1 に変えられる階層を探す
                if dir == 0 {
                    // dir=1 に変更
                    levels[byte_i] |= 1 << (bitpos * 2);

                    // それより下は全て削る（valid=0 にする）
                    for bi in byte_i..levels.len() {
                        // 下位階層だけ 2bit = 00 にする
                        let mut b = levels[bi];
                        for bp in (0..4).rev() {
                            if bi == byte_i && bp <= bitpos {
                                continue;
                            }
                            // valid=0 dir=0 をセット
                            let mask = 0b11 << (bp * 2);
                            b &= !mask;
                        }
                        levels[bi] = b;
                    }

                    return BitVec(levels);
                }
            }
        }

        // 右側が存在しない（＝一番右の葉）なら
        // artificial な最大値として 0xFF を追加する
        levels.push(0xFF);
        BitVec(levels)
    }
}
