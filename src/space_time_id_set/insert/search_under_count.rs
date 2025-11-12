use std::collections::BTreeMap;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, SpaceTimeIdSet},
};
impl SpaceTimeIdSet {
    ///与えられた次元において、下位の範囲の個数を読み取る
    pub fn search_under_count(btree: &BTreeMap<BitVec, LayerInfo>, encoded: &BitVec) -> usize {
        match btree.get(&encoded) {
            Some(v) => v.count,
            None => 0,
        }
    }
}
