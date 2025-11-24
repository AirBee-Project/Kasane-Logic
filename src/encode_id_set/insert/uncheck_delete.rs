use crate::encode_id_set::{BTreeMap, EncodeIDSet, HashSet, Index};
use crate::{bit_vec::BitVec, encode_id_set::LayerInfo};

impl EncodeIDSet {
    pub(crate) fn uncheck_delete(&mut self, index: &Index) {
        let removed = self
            .reverse
            .remove(index)
            .expect("Internal error: reverse index not found in uncheck_delete");

        Self::update_layer_delete(&mut self.f, &removed.f, *index);
        Self::update_layer_delete(&mut self.x, &removed.x, *index);
        Self::update_layer_delete(&mut self.y, &removed.y, *index);
    }

    ///上位の階層のcountから-1
    fn update_layer_delete(map: &mut BTreeMap<BitVec, LayerInfo>, key: &BitVec, index: usize) {
        for key_top in key.ancestors() {
            if key_top == *key {
                map.entry(key_top)
                    .and_modify(|v| {
                        v.count -= 1;
                        v.index.remove(&index);
                    })
                    .or_insert(LayerInfo {
                        index: HashSet::from([index]),
                        count: 1,
                    });
            } else {
                map.entry(key_top)
                    .and_modify(|v| {
                        v.count -= 1;
                    })
                    .or_insert(LayerInfo {
                        index: HashSet::from([]),
                        count: 0,
                    });
            }
        }
    }
}
