use crate::{
    bit_vec::BitVec,
    space_time_id_set::{
        Index, ReverseInfo, SpaceTimeIdSet, insert::select_dimensions::DimensionSelect,
    },
};
#[derive(Debug)]
pub struct RangesCollect {
    pub main: Vec<BitVec>,
    pub a: Vec<BitVec>,
    pub b: Vec<BitVec>,
}

impl SpaceTimeIdSet {
    ///下位,下位,上位の場合に自身を切断する
    pub(crate) fn split_self(
        &self,
        divison: &mut RangesCollect,
        target_bit_index: Index,
        target_dim: &DimensionSelect,
    ) {
        let reverse = self
            .reverse
            .get(&target_bit_index)
            .expect("Internal error: reverse index not found in under_under_top");

        match target_dim {
            DimensionSelect::F => {
                divison.main.push(reverse.f.clone());
            }
            DimensionSelect::X => {
                divison.main.push(reverse.x.clone());
            }
            DimensionSelect::Y => {
                divison.main.push(reverse.y.clone());
            }
        }
    }
}
