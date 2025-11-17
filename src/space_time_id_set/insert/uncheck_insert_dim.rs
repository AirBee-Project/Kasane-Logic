use crate::{
    bit_vec::BitVec,
    space_time_id_set::{SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

impl SpaceTimeIdSet {
    pub fn uncheck_insert_dim(
        &mut self,
        dim_select: MainDimensionSelect,
        main: &BitVec,
        a: &BitVec,
        b: &BitVec,
    ) {
        match dim_select {
            MainDimensionSelect::F => {
                self.uncheck_insert(main, a, b);
            }
            MainDimensionSelect::X => {
                self.uncheck_insert(a, main, b);
            }
            MainDimensionSelect::Y => {
                self.uncheck_insert(a, b, main);
            }
        }
    }
}
