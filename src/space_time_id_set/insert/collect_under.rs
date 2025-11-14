use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

use std::ops::Bound::Excluded;

impl SpaceTimeIdSet {
    ///与えられた次元において、下位の範囲を収集する
    pub fn collect_under(
        &self,
        main_bit: &BitVec,
        main_dim_select: &MainDimensionSelect,
    ) -> Vec<Index> {
        let mut main_under = Vec::new();

        let dims = self.select_dimensions(&main_dim_select);

        let range = main_bit.under_prefix();

        for (_, layerinfo) in dims.main.range((Excluded(range.0), Excluded(range.1))) {
            main_under.extend(layerinfo.index.clone());
        }

        Vec::from_iter(main_under)
    }
}
