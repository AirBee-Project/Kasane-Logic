use std::collections::{BTreeMap, HashSet};

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, ReverseInfo, SpaceTimeIdSet},
};

//Todo:ここら辺に隣と結合する処理を追加したい

impl SpaceTimeIdSet {
    /// 検証なしでIDをセットに挿入する
    ///
    /// 各次元のBitVecを直接挿入する。包含関係のチェックは行わない。
    ///
    /// # 引数
    /// * `f` - F次元のBitVec
    /// * `x` - X次元のBitVec
    /// * `y` - Y次元のBitVec
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

    /// 上位の階層のカウントを+1する
    ///
    /// 指定されたBitVecとその全ての上位階層の情報を更新する。
    ///
    /// # 引数
    /// * `map` - 更新対象のBTreeMap
    /// * `key` - 挿入するBitVec
    /// * `index` - IDのインデックス
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
                        count: 0,
                    });
            }
        }
    }
}
