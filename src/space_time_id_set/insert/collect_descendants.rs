use crate::{
    bit_vec::BitVec,
    space_time_id_set::{EncodeIDSet, Index, insert::select_dimensions::DimensionSelect},
};

use std::ops::Bound::Excluded;

impl EncodeIDSet {
    /// 指定された次元において、自分が含む子孫のインデックスを収集する
    pub(crate) fn collect_descendants(
        &self,
        main_bit: &BitVec,
        main_dim: &DimensionSelect,
    ) -> Vec<Index> {
        println!("MAINBIT:{}", main_bit);

        let mut main_descendants = Vec::new();

        let dims = self.dims_btree(&main_dim);

        for (_, layerinfo) in dims
            .main
            .range((Excluded(main_bit.clone()), Excluded(main_bit.upper_bound())))
        {
            main_descendants.extend(layerinfo.index.clone());
        }

        println!("{:?}", main_descendants);

        main_descendants
    }
}
