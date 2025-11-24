use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    space_time_id_set::{EncodeIDSet, Index, insert::select_dimensions::DimensionSelect},
};

impl EncodeIDSet {
    ///上位,上位,下位の場合に相手を切断する
    pub(crate) fn split_other(
        &self,
        target_index: Index,
        target_bit: &BitVec,
        target_dim: &DimensionSelect,
        need_delete: &mut HashSet<Index>,
        need_insert: &mut HashSet<EncodeID>,
    ) {
        let reverse = self
            .reverse
            .get(&target_index)
            .expect("Internal error: reverse index not found in top_top_under");

        let top = match target_dim {
            DimensionSelect::F => reverse.f.clone(),
            DimensionSelect::X => reverse.x.clone(),
            DimensionSelect::Y => reverse.y.clone(),
        };

        let splited = top.subtract_range(&target_bit);

        let reverse_f = reverse.f.clone();
        let reverse_x = reverse.x.clone();
        let reverse_y = reverse.y.clone();

        for single in splited {
            match target_dim {
                DimensionSelect::F => need_insert.insert(EncodeID {
                    f: single,
                    x: reverse_x.clone(),
                    y: reverse_y.clone(),
                }),
                DimensionSelect::X => need_insert.insert(EncodeID {
                    f: reverse_f.clone(),
                    x: single,
                    y: reverse_y.clone(),
                }),
                DimensionSelect::Y => need_insert.insert(EncodeID {
                    f: reverse_f.clone(),
                    x: reverse_x.clone(),
                    y: single,
                }),
            };
        }

        need_delete.insert(target_index);
    }
}
