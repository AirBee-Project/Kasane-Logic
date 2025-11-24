use crate::{
    bit_vec::BitVec,
    encode_id_set::{EncodeIDSet, Index, insert::select_dimensions::DimensionSelect},
};
#[derive(Debug)]
pub struct RangesCollect {
    pub f: Vec<BitVec>,
    pub x: Vec<BitVec>,
    pub y: Vec<BitVec>,
}

impl EncodeIDSet {
    ///下位,下位,上位の場合に自身を切断する
    pub(crate) fn split_self(
        &self,
        divison_collect: &mut RangesCollect,
        target_index: Index,
        target_dim: &DimensionSelect,
    ) {
        let reverse = self
            .reverse
            .get(&target_index)
            .expect("Internal error: reverse index not found in under_under_top");

        match target_dim {
            DimensionSelect::F => {
                divison_collect.f.push(reverse.f.clone());
            }
            DimensionSelect::X => {
                divison_collect.x.push(reverse.x.clone());
            }
            DimensionSelect::Y => {
                divison_collect.y.push(reverse.y.clone());
            }
        }
    }
}
