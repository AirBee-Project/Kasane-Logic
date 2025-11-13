use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

impl SpaceTimeIdSet {
    /// 与えられた次元において、上位の範囲を収集する
    pub fn collect_top(
        &self,
        main_bit: &BitVec,
        main_dim_select: &MainDimensionSelect,
    ) -> HashSet<Index> {
        //この関数が正しい範囲を返していない

        println!("===========collect_top===========");

        let dims = self.select_dimensions(&main_dim_select);

        println!("INPUT : {}", main_bit);
        for ele in dims.main {
            print!("{}", ele.0);
            println!("{:?}", ele.1);
        }

        let mut result = HashSet::new();

        println!("-----------");

        for top in main_bit.top_prefix() {
            if let Some(v) = dims.main.get(&top) {
                print!("{}", top);
                println!("{:?}", v);
                result.extend(v.index.iter().copied());
            }
        }

        println!("===========collect_top===========");

        result
    }
}
