use crate::bit_vec::BitVec;

impl BitVec {
    /// 最下層（最後のバイト）にある有効な2ビット層を 00 にリセットする
    pub fn remove_bottom_layer(&mut self) {
        if let Some(last) = self.0.last_mut() {
            for i in 0..=3 {
                // 2ビットマスクを作成
                let mask = 0b00000011 << (i * 2);
                let masked = *last & mask;

                // 有効な階層（00 以外）だった場合、その2ビットを 00 にする
                if masked != 0 {
                    // 該当ビットを消去
                    *last = *last & !mask;

                    //もし最後のu8が空の場合はu8をVecから削除
                    if *last == 0 {
                        self.0.pop();
                    }

                    break;
                }
            }
        }
    }
}
