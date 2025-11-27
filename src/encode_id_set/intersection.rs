use crate::encode_id_set::EncodeIDSet;

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
        let mut result = EncodeIDSet::new();

        //個別に処理を行う
        for (_, reverse) in &small.reverse {
            for encode_id in large.get(reverse) {
                result.uncheck_insert(encode_id);
            }
        }

        result
    }
}
