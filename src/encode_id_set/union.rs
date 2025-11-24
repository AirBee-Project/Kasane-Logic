use crate::encode_id_set::{EncodeIDSet, insert::select_dimensions::DimensionSelect};

impl EncodeIDSet {
    /// 二つのSpaceTimeIDSetを結合する
    pub fn union(&self, other: &EncodeIDSet) -> EncodeIDSet {
        // IDの個数が少ないほうを small、多いほうを large にする
        let (small, large) = if self.iter().len() <= other.iter().len() {
            (self, other)
        } else {
            (other, self)
        };

        // large をコピーして small の内容を挿入する
        let mut result = large.clone();
        for (_, reverse) in small.reverse.clone() {
            result.uncheck_insert(reverse);
        }

        result
    }
}
