use crate::bit_vec::BitVec;

impl BitVec {
    /// 最下層の各2ビットペアを反転する:
    /// - `10` → `11`
    /// - `11` → `10`
    pub fn reverse_bottom_layer(&mut self) {
        if let Some(last) = self.0.last_mut() {
            for i in 0..=3 {
                // 2ビットマスクを作成
                let mask = 0b00000011 << (i * 2);
                let masked = *last & mask;

                match masked {
                    v if v == (0b00000010 << (i * 2)) => {
                        // 10 -> 11
                        *last |= 0b00000001 << (i * 2);
                        break;
                    }
                    v if v == (0b00000011 << (i * 2)) => {
                        // 11 -> 10
                        *last ^= 0b00000001 << (i * 2);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
