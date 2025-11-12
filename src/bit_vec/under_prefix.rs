use crate::bit_vec::BitVec;

impl BitVec {
    ///下位範囲を検索するときに必要な右側の端点の値を出す
    pub fn under_prefix(&self) -> BitVec {
        let mut copy = self.0.clone();

        // 逆順でミュータブル参照を回す
        'outer: for byte in copy.iter_mut().rev() {
            for i in 0..=3 {
                let mask: u8 = 0b00000011 << (i * 2);

                if *byte & mask == 0 {
                    //無効な階層
                    continue;
                } else {
                    // 有効な階層
                    if (*byte & mask) == (0b00000010 << (i * 2)) {
                        // 分岐Bitが0の場合 → 1にする
                        *byte = *byte | 0b00000011 << (i * 2);
                        break 'outer;
                    } else {
                        // 分岐Bitが1の場合 → 0にする
                        *byte ^= 0b00000001 << (i * 2);
                    }
                }
            }
        }

        BitVec(copy)
    }
}
