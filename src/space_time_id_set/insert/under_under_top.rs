use std::{collections::HashSet, ops::Div};

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect},
};
#[derive(Debug)]
pub struct NeedDivison {
    pub f: Vec<BitVec>,
    pub x: Vec<BitVec>,
    pub y: Vec<BitVec>,
}

impl SpaceTimeIdSet {
    ///自分を切断する
    pub fn under_under_top(
        &self,
        divison: &mut NeedDivison,
        target_bit_index: Index,
        target_dim: DimensionSelect,
    ) {
        let reverse = self.reverse.get(&target_bit_index).unwrap();

        match target_dim {
            DimensionSelect::F => {
                divison.f.push(reverse.f.clone());
            }
            DimensionSelect::X => {
                divison.x.push(reverse.x.clone());
            }
            DimensionSelect::Y => {
                divison.y.push(reverse.y.clone());
            }
        }
    }
}
