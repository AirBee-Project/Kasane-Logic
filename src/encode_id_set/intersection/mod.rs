use crate::encode_id_set::{EncodeIDSet, Index};

impl EncodeIDSet {
    /// 二つのSpaceTimeIDSetを結合する
    pub fn intersection(&self, other: &EncodeIDSet) -> EncodeIDSet {
        // 小さい方を small、大きい方を large にする
        let (small, large) = if self.iter().len() <= other.iter().len() {
            (self, other)
        } else {
            (other, self)
        };

        // large をコピーして small の内容を挿入する
        let mut result = large.clone();

        //個別に処理を行う
        for (index, reverse) in &small.reverse {
            //

            //reverse要素を探索する
        }

        result
    }
}
