use crate::bit_vec::BitVec;

impl BitVec {
    ///あるBitVecが表す範囲より上位の範囲を表すBitVecを全て返す
    pub fn top_prefix(&self) -> Vec<BitVec> {
        let mut result: Vec<BitVec> = vec![];
        for byte in &self.0 {
            for i in 0..4 {
                //今着目している階層の2Bitだけが有効になっている
                let masked: u8 = byte & (0b11000000 >> 2 * i);

                //終了条件
                if masked == 0 {
                    break;
                }

                match result.last() {
                    Some(v) => {
                        let mut copy = v.clone();
                        if i == 0 {
                            copy.0.push(masked);
                        } else {
                            //今のu8に付け足す場合
                            if let Some(last) = copy.0.last_mut() {
                                *last = *last | masked;
                            }
                        }
                        result.push(copy);
                    }
                    None => {
                        result.push(BitVec(vec![masked]));
                    }
                }
            }
        }

        result
    }
}
