use std::collections::{BTreeMap, HashSet};

use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    encode_id_set::{EncodeIDSet, LayerInfo},
};

impl EncodeIDSet {
    pub(crate) fn uncheck_insert(&mut self, encode_id: EncodeID) {
        let index = self.generate_index();

        Self::update_layer(&mut self.f, &encode_id.f, index);
        Self::update_layer(&mut self.x, &encode_id.x, index);
        Self::update_layer(&mut self.y, &encode_id.y, index);

        self.reverse.insert(index, encode_id);
    }

    ///上位の階層のcountに+1
    fn update_layer(map: &mut BTreeMap<BitVec, LayerInfo>, key: &BitVec, index: usize) {
        for key_top in key.ancestors() {
            if key_top == *key {
                map.entry(key_top)
                    .and_modify(|v| {
                        v.count += 1;
                        v.index.insert(index);
                    })
                    .or_insert(LayerInfo {
                        index: HashSet::from([index]),
                        count: 1,
                    });
            } else {
                map.entry(key_top)
                    .and_modify(|v| {
                        v.count += 1;
                    })
                    .or_insert(LayerInfo {
                        index: HashSet::from([]),
                        count: 1,
                    });
            }
        }
    }
}
