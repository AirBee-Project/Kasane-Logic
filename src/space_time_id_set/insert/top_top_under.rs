use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect},
};

impl SpaceTimeIdSet {
    ///相手を切断する
    pub fn top_top_under(
        &mut self,
        target_index: Index,
        target_bit: BitVec,
        target_dim: DimensionSelect,
        need_delete: &mut HashSet<Index>,
    ) {
        // 既に削除予定なら何もしない
        if need_delete.contains(&target_index) {
            return;
        }

        // reverseから必要なデータをcloneして借用を解放
        let reverse = self.reverse.get(&target_index).unwrap();

        let top = match target_dim {
            DimensionSelect::F => reverse.f.clone(),
            DimensionSelect::X => reverse.x.clone(),
            DimensionSelect::Y => reverse.y.clone(),
        };

        let splited = BitVec::division(top, vec![target_bit]);

        // ここでreverseのフィールドを個別にclone
        let reverse_f = reverse.f.clone();
        let reverse_x = reverse.x.clone();
        let reverse_y = reverse.y.clone();

        for single in splited {
            match target_dim {
                DimensionSelect::F => self.uncheck_insert(&single, &reverse_x, &reverse_y),
                DimensionSelect::X => self.uncheck_insert(&reverse_f, &single, &reverse_y),
                DimensionSelect::Y => self.uncheck_insert(&reverse_f, &reverse_x, &single),
            }
        }

        need_delete.insert(target_index);
    }
}
