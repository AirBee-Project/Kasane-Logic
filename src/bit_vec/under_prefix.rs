use crate::bit_vec::BitVec;

impl BitVec {
    /// あるBitVecが表す範囲より下位の範囲の開始と終了を返す
    ///
    /// 階層構造において、この範囲に包含される全ての下位範囲の
    /// 開始位置と終了位置を返す（両端を含む）。
    ///
    /// # 戻り値
    /// (開始位置, 終了位置) のタプル
    pub fn under_prefix(&self) -> (BitVec, BitVec) {
        if self.clone() > self.next_prefix() {
            println!("SELF  :{}", self.clone());
            println!("UNDER :{}", self.next_prefix());
            panic!()
        }
        (self.clone(), self.next_prefix())
    }

    /// 次の接頭辞（prefix）を計算する
    ///
    /// 階層構造において、現在の範囲の次に来る範囲を表すBitVecを返す。
    pub fn next_prefix(&self) -> BitVec {
        let mut copyed = self.clone();
        let len = copyed.0.len();

        // 全ての分岐Bitが 11 の場合のみ true のまま残る
        let mut all_one = true;

        // まず "全ての分岐Bitが 11 かどうか" を判定
        for (_byte_index, byte) in self.0.iter().enumerate().rev() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);

                // 最後の2bit(i == 0) もここでは判定に含める
                if (byte & mask) >> (i * 2) != 0b11 {
                    all_one = false;
                    break;
                }
            }
            if !all_one {
                break;
            }
        }

        // ここから next_prefix 本体
        for (byte_index, byte) in copyed.0.iter_mut().enumerate().rev() {
            for i in 0..=3 {
                // 最後の2bit（i == 0）だけ特別処理
                if byte_index == len - 1 && i == 0 {
                    if all_one {
                        // 全て 11 のときだけ 11 -> 10 に変える
                        *byte = (*byte & !(0b11)) | 0b10;
                    }
                    continue;
                }

                let mask = 0b00000011 << (i * 2);
                let masked = *byte & mask;

                match masked >> (i * 2) {
                    0b10 => {
                        // 10 -> 11
                        *byte |= 0b01 << (i * 2);
                        return copyed;
                    }
                    0b11 => {
                        // 11 -> 10
                        *byte ^= 0b11 << (i * 2);
                        // → 続行して上位で処理させる
                    }
                    _ => {}
                }
            }
        }

        copyed
    }
}
