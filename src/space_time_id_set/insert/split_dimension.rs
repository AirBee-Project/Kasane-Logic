use crate::{bit_vec::BitVec, space_time_id_set::SpaceTimeIdSet};

impl SpaceTimeIdSet {
    pub fn split_dimension(top: &BitVec, under: &mut BitVec) -> Vec<BitVec> {
        let mut result: Vec<BitVec> = vec![];

        while top != under {
            under.reverse_bottom_layer();

            result.push(under.clone());

            under.remove_bottom_layer();
        }

        result
    }
}
