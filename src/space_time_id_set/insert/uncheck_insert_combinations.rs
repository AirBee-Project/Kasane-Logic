use crate::{
    bit_vec::BitVec,
    space_time_id_set::{SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

impl SpaceTimeIdSet {
    /// 他次元との組み合わせで uncheck_insert を呼ぶ
    pub fn uncheck_insert_combinations(
        &mut self,
        dim_sel: &MainDimensionSelect,
        main: &BitVec,
        others: &[&Vec<(usize, BitVec)>; 2],
    ) {
        // 組み合わせを生成して挿入
        for (_, b1) in &*others[0] {
            for (_, b2) in &*others[1] {
                match dim_sel {
                    MainDimensionSelect::F => self.uncheck_insert(main, b1, b2),
                    MainDimensionSelect::X => self.uncheck_insert(b1, main, b2),
                    MainDimensionSelect::Y => self.uncheck_insert(b1, b2, main),
                }
            }
        }
    }
}
