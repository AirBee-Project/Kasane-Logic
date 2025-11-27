use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    encode_id_set::{EncodeIDSet, utils::select_dimensions::DimensionSelect},
};
#[derive(Debug)]
pub struct RangesCollect {
    pub f: Vec<BitVec>,
    pub x: Vec<BitVec>,
    pub y: Vec<BitVec>,
    pub t: Vec<BitVec>,
}

impl EncodeIDSet {
    ///下位,下位,上位の場合に自身を切断する
    pub(crate) fn split_self(
        &self,
        target_reverse: &EncodeID,
        divison_collect: &mut RangesCollect,
        target_dim: &DimensionSelect,
    ) {
        match target_dim {
            DimensionSelect::F => {
                divison_collect.f.push(target_reverse.f.clone());
            }
            DimensionSelect::X => {
                divison_collect.x.push(target_reverse.x.clone());
            }
            DimensionSelect::Y => {
                divison_collect.y.push(target_reverse.y.clone());
            }
            DimensionSelect::T => {
                divison_collect.t.push(target_reverse.t.clone());
            }
        }
    }
}
