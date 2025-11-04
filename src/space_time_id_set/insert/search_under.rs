use std::collections::BTreeMap;

use crate::{
    space_time_id_set::{LayerInfo, SpaceTimeIdSet},
    r#type::bit_vec::{self, BitVec},
};
impl SpaceTimeIdSet {
    pub fn search_under(btree: &BTreeMap<BitVec, LayerInfo>, encoded: &Vec<BitVec>) -> Vec<usize> {
        let mut result = vec![];
        for bit_vec in encoded {
            match btree.get(&bit_vec) {
                Some(v) => {
                    result.push(v.count);
                }
                None => {
                    result.push(0);
                }
            }
        }
        result
    }
}
