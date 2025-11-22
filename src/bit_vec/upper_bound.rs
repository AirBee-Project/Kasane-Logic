use crate::bit_vec::BitVec;

impl BitVec {
    /// # 概要
    /// この関数は、`BitVec` が表す階層ID（2ビット単位の階層構造）について、
    /// 同一の prefix に属する範囲の **右側開区間上限（upper bound）** を計算して返します。
    ///
    /// # 動作例
    /// - 入力 `1010111011000000`→ 出力 `10101111`
    /// - 入力 `11101000`→ 出力 `11101100`
    pub fn upper_bound(&self) -> BitVec {
        let mut copyed = self.clone();

        // upper_bound 本体（2bit単位で後ろから走査）
        for (_byte_index, byte) in copyed.0.iter_mut().enumerate().rev() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);
                let masked = *byte & mask;

                match masked >> (i * 2) {
                    0b10 => {
                        // 10 -> 11 で終了
                        *byte |= 0b01 << (i * 2);

                        // 末尾の空バイト削除
                        while let Some(&last) = copyed.0.last() {
                            if last == 0 {
                                copyed.0.pop();
                            } else {
                                break;
                            }
                        }
                        return copyed;
                    }
                    0b11 => {
                        // 11 -> 10 に戻して繰り上げ継続
                        *byte ^= 0b11 << (i * 2);
                    }
                    _ => {}
                }
            }
        }

        // ここでも末尾の空バイト削除
        while let Some(&last) = copyed.0.last() {
            if last == 0 {
                copyed.0.pop();
            } else {
                break;
            }
        }

        copyed
    }
}
