use std::collections::{BTreeMap, HashSet};

use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    space_time_id_set::{LayerInfo, SpaceTimeIDSet, insert::select_dimensions::DimensionSelect},
};

impl SpaceTimeIDSet {
    pub(crate) fn uncheck_insert(
        &mut self,
        main: &BitVec,
        a: &BitVec,
        b: &BitVec,
        main_dim_select: &DimensionSelect,
    ) {
        let index = self.generate_index();
        let mut dims = self.dims_btree_mut(main_dim_select);
        let map = Self::map_dims(main, a, b, main_dim_select);

        Self::update_layer(&mut dims.main, main, index);
        Self::update_layer(&mut dims.a, a, index);
        Self::update_layer(&mut dims.b, b, index);

        self.reverse.insert(
            index,
            EncodeID {
                f: map.f.clone(),
                x: map.x.clone(),
                y: map.y.clone(),
            },
        );
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
