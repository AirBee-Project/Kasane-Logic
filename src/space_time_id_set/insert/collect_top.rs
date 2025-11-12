use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

impl SpaceTimeIdSet {
    /// 与えられた次元において、上位の範囲を収集する
    pub fn collect_top(
        &self,
        main_bit: &BitVec,
        main_dim_select: &MainDimensionSelect,
    ) -> HashSet<Index> {
        let dims = self.select_dimensions(&main_dim_select);

        let mut result = HashSet::new();
        for top in main_bit.top_prefix() {
            if let Some(v) = dims.main.get(&top) {
                result.extend(v.index.iter().copied());
            }
        }

        result
    }
}
