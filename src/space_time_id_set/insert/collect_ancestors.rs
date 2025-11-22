use std::collections::HashSet;

use crate::{
    bit_vec::HierarchicalKey,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::select_dimensions::DimensionSelect},
};

impl SpaceTimeIdSet {
    /// 指定された次元において、自分を含む祖先のインデックスを収集する
    pub(crate) fn collect_ancestors(
        &self,
        main_bit: &HierarchicalKey,
        main_dim_select: &DimensionSelect,
    ) -> Vec<Index> {
        let dims = self.dims_btree(main_dim_select);

        let mut result = HashSet::new();

        for top in main_bit.ancestors() {
            if let Some(v) = dims.main.get(&top) {
                result.extend(v.index.iter().copied());
            }
        }

        Vec::from_iter(result)
    }
}
