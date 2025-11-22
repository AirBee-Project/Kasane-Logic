use crate::bit_vec::HierarchicalKey;

impl HierarchicalKey {
    /// 最下層の有効な2ビット階層を取り除く
    ///
    /// - 最下層の最初に見つかった有効階層（00以外）を 00 にリセット
    /// - もし最後のu8が空になった場合、Vecから削除
    pub fn pop_leaf(&mut self) {
        if let Some(last) = self.0.last_mut() {
            for i in 0..=3 {
                let mask = 0b00000011 << (i * 2);
                let masked = *last & mask;

                if masked != 0 {
                    *last &= !mask;

                    if *last == 0 {
                        self.0.pop();
                    }

                    break;
                }
            }
        }
    }
}
