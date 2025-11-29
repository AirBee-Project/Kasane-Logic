use crate::{encode_id::EncodeID, encode_id_set::EncodeIDSet, space_time_id::SpaceTimeID};

impl EncodeIDSet {
    pub fn decode(self) -> Vec<SpaceTimeID> {
        let mut result = vec![];

        for encode_id in self.iter() {
            result.push(encode_id.decode());
        }

        result
    }
}
