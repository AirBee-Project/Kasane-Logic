use std::collections::BTreeMap;

use crate::{
    space_time_id_set::{LayerInfo, SpaceTimeIdSet},
    r#type::bit_vec::BitVec,
};

impl SpaceTimeIdSet {
    pub fn tmp(
        main_dim: &BTreeMap<BitVec, LayerInfo>,
        other_dim: &[&BTreeMap<BitVec, LayerInfo>; 2],
        min_under: &usize,
        main_encoded: &BitVec,
    ) {
        //まず上位IDを検索
        //上位には同位を含む

        //上位のIDを生成する

        //下位の検索が必要なのか？

        //必要な場合は検索
    }
}
