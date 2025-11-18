use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect},
};

use std::ops::Bound::Excluded;

impl SpaceTimeIdSet {
    /// 与えられた次元において、下位の範囲を収集する
    ///
    /// 指定されたBitVecに包含される全ての下位範囲のインデックスを返す。
    ///
    /// # 引数
    /// * `main_bit` - 対象のBitVec
    /// * `main_dim_select` - 対象次元（F, X, Y のいずれか）
    pub fn collect_under(
        &self,
        main_bit: &BitVec,
        main_dim_select: &DimensionSelect,
    ) -> Vec<Index> {
        let mut main_under = Vec::new();

        let dims = self.select_dimensions(main_dim_select);

        let range = main_bit.under_prefix();

        for (_, layerinfo) in dims.main.range((Excluded(range.0), Excluded(range.1))) {
            main_under.extend(layerinfo.index.clone());
        }

        Vec::from_iter(main_under)
    }
}
