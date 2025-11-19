use std::collections::{BTreeMap, HashSet};

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, ReverseInfo, SpaceTimeIdSet},
};

//Todo:ここら辺に隣と結合する処理を追加したい

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
        for key_top in key.top_prefix() {
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
