use crate::bit_vec::BitVec;

impl BitVec {
    /// (self, next_prefix)
    pub fn under_prefix(&self) -> (BitVec, BitVec) {
        if self.clone() > self.next_prefix() {
            println!("SELF  :{}", self.clone());
            println!("UNDER :{}", self.next_prefix());
            panic!()
        }
        (self.clone(), self.next_prefix())
    }

    pub fn next_prefix(&self) -> BitVec {
        let mut copyed = self.clone();

        // ここから next_prefix 本体
        for (byte_index, byte) in copyed.0.iter_mut().enumerate().rev() {
            for i in 0..=3 {
                // // 最後の2bit（i == 0）だけ特別処理
                // if byte_index == len - 1 && i == 0 {
                //     continue;
                // }

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
                        *byte ^= 0b01 << (i * 2);
                        // → 続行して上位で処理させる
                    }
                    _ => {}
                }
            }
        }

        copyed
    }
}
