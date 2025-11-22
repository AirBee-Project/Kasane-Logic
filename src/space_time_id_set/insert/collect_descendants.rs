use crate::{
    bit_vec::HierarchicalKey,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::select_dimensions::DimensionSelect},
};

use std::ops::Bound::Excluded;

impl SpaceTimeIdSet {
    /// 指定された次元において、自分が含む子孫のインデックスを収集する
    pub(crate) fn collect_descendants(
        &self,
        main_bit: &HierarchicalKey,
        main_dim_select: &DimensionSelect,
    ) -> Vec<Index> {
        let mut main_under = Vec::new();

        let dims = self.dims_btree(&main_dim_select);

        for (_, layerinfo) in dims
            .main
            .range((Excluded(main_bit.clone()), Excluded(main_bit.upper_bound())))
        {
            main_under.extend(layerinfo.index.clone());
        }

        main_under
    }
}
