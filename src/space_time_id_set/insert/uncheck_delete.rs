use crate::space_time_id_set::{BTreeMap, HashMap, HashSet, Index, SpaceTimeIdSet};
use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, ReverseInfo},
};

impl SpaceTimeIdSet {
    pub fn uncheck_delete(&mut self, index: &Index) {
        println!("DELETE:{}", index);
        let removed = self.reverse.remove(index).unwrap();

        Self::update_layer_delete(&mut self.f, &removed.f, *index);
        Self::update_layer_delete(&mut self.x, &removed.x, *index);
        Self::update_layer_delete(&mut self.y, &removed.y, *index);
    }

    ///上位の階層のcountから-1
    fn update_layer_delete(map: &mut BTreeMap<BitVec, LayerInfo>, key: &BitVec, index: usize) {
        for key_top in key.top_prefix() {
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
