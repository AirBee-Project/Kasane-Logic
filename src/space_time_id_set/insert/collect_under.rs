use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect},
};

use std::ops::Bound::Excluded;

impl SpaceTimeIdSet {
    ///与えられた次元において、下位の範囲を収集する
    pub(crate) fn collect_under(
        &self,
        main_bit: &BitVec,
        main_dim_select: &DimensionSelect,
    ) -> Vec<Index> {
        let mut main_under = Vec::new();

        let dims = self.select_dimensions(&main_dim_select);

        for (_, layerinfo) in dims
            .main
            .range((Excluded(main_bit.clone()), Excluded(main_bit.upper_bound())))
        {
            main_under.extend(layerinfo.index.clone());
        }

        main_under
    }
}
