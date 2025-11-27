use crate::encode_id_map::{EncodeIDMap, Index};

impl<V> EncodeIDMap<V> {
    pub(crate) fn generate_index(&mut self) -> Index {
        self.index += 1;
        self.index
    }
}
