use crate::encode_id_set::{EncodeIDSet, Index};

impl EncodeIDSet {
    pub(crate) fn generate_index(&mut self) -> Index {
        self.index = self.index + 1;
        self.index.clone()
    }
}
