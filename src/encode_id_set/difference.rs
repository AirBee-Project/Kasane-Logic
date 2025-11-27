use crate::encode_id_set::EncodeIDSet;

impl EncodeIDSet {
    pub fn difference(&self, other: &EncodeIDSet) -> EncodeIDSet {
        let (small, large) = if self.iter().len() <= other.iter().len() {
            (self, other)
        } else {
            (other, self)
        };

        // large をコピーして small の内容を削除するする
        let mut result = large.clone();
        for (_, reverse) in small.reverse.clone() {
            result.remove(reverse);
        }

        result
    }
}
