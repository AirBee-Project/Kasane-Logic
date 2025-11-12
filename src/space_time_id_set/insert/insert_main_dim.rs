use crate::{bit_vec::BitVec, space_time_id_set::SpaceTimeIdSet};

#[derive(Clone, Copy, Debug)]
pub enum MainDimensionSelect {
    F,
    X,
    Y,
}

impl MainDimensionSelect {
    pub fn as_index(&self) -> usize {
        match self {
            MainDimensionSelect::F => 0,
            MainDimensionSelect::X => 1,
            MainDimensionSelect::Y => 2,
        }
    }
}

impl SpaceTimeIdSet {
    /// 代表次元×他の次元を挿入処理する
    pub fn insert_main_dim(
        &mut self,
        main_bit: &BitVec,
        main_index: &usize,
        main_under_count: &usize,
        main_encoded: &mut Vec<(usize, BitVec)>,
        other_encoded: &[&Vec<(usize, BitVec)>; 2],
        main_dim_select: MainDimensionSelect,
    ) {
        //代表次元における上位範囲を収拾する

        let main_top = Self::collect_top(&self, main_bit, &main_dim_select);

        //代表次元において、上位も下位も存在しなかった場合
        if main_top.is_empty() && *main_under_count == 0 {
            //代表次元をBitVecから削除
            let _removed = main_encoded.remove(*main_index);
            //挿入
            self.uncheck_insert_combinations(&main_dim_select, main_bit, other_encoded);
            return;
        }

        //main_topから検索して処理していく
        match self.scan_and_insert_top(
            main_bit,
            &main_top,
            other_encoded,
            main_dim_select,
            main_under_count,
        ) {
            super::scan_and_insert_top::ResultTop::End => {
                //この階層はここで終了でよい
                main_encoded.remove(*main_index);
                return;
            }
            super::scan_and_insert_top::ResultTop::Continue => {
                //処理を続行する
            }
        };

        //代表次元における下位範囲を収拾する
        let main_under = self.collect_under(main_bit, &main_dim_select);

        self.scan_and_insert_under(main_bit, &main_under, other_encoded, main_dim_select);
        main_encoded.remove(*main_index);

        return;
    }
}
