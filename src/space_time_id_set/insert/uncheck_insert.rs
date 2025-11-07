use std::collections::{BTreeMap, HashSet};

use crate::{
    space_time_id_set::{LayerInfo, ReverseInfo, SpaceTimeIdSet},
    r#type::bit_vec::BitVec,
};

impl SpaceTimeIdSet {
    pub fn uncheck_insert(&mut self, f: &BitVec, x: &BitVec, y: &BitVec) {
        let index = self.generate_index();

        // 各次元に共通処理を適用
        Self::update_layer(&mut self.f, f, index);
        Self::update_layer(&mut self.x, x, index);
        Self::update_layer(&mut self.y, y, index);

        //逆引きに挿入
        self.reverse.insert(
            index,
            ReverseInfo {
                f: f.clone(),
                x: x.clone(),
                y: y.clone(),
            },
        );
    }

    ///上位の階層のcountに+1
    fn update_layer(map: &mut BTreeMap<BitVec, LayerInfo>, key: &BitVec, index: usize) {
        map.entry(key.clone())
            .and_modify(|v| {
                v.index.insert(index);
            })
            .or_insert(LayerInfo {
                index: HashSet::from([index]),
                count: 0,
            });

        while let Some(top) = key.top_prefix().pop() {
            map.entry(top)
                .and_modify(|v| v.count += 1)
                .or_insert(LayerInfo {
                    index: HashSet::from([index]),
                    count: 0,
                });
        }
    }
}
