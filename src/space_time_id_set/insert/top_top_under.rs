use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{
        Index, ReverseInfo, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect,
    },
};

impl SpaceTimeIdSet {
    /// 相手を切断する
    ///
    /// 既存のIDの特定の次元を分割して、指定されたBitVecを除外する。
    /// 分割後の新しいIDは need_insert に追加され、元のIDは need_delete に追加される。
    ///
    /// # 引数
    /// * `target_index` - 分割対象のIDのインデックス
    /// * `target_bit` - 除外するBitVec
    /// * `target_dim` - 分割する次元（F, X, Y のいずれか）
    /// * `need_delete` - 削除が必要なインデックスを格納するセット
    /// * `need_insert` - 挿入が必要なReverseInfoを格納するセット
    pub fn top_top_under(
        &mut self,
        target_index: Index,
        target_bit: BitVec,
        target_dim: DimensionSelect,
        need_delete: &mut HashSet<Index>,
        need_insert: &mut HashSet<ReverseInfo>,
    ) {
        println!("top_top_under");

        // reverseから必要なデータをcloneして借用を解放
        let reverse = self.reverse.get(&target_index).unwrap();

        let top = match target_dim {
            DimensionSelect::F => reverse.f.clone(),
            DimensionSelect::X => reverse.x.clone(),
            DimensionSelect::Y => reverse.y.clone(),
        };

        println!("DIM{:?}", target_dim);

        println!("TOP{}", top);
        println!("TAR{}", target_bit);

        let splited = BitVec::division(top, vec![target_bit]);

        println!("splited:{:?}", splited);

        // ここでreverseのフィールドを個別にclone
        let reverse_f = reverse.f.clone();
        let reverse_x = reverse.x.clone();
        let reverse_y = reverse.y.clone();

        for single in splited {
            match target_dim {
                DimensionSelect::F => need_insert.insert(ReverseInfo {
                    f: single,
                    x: reverse_x.clone(),
                    y: reverse_y.clone(),
                }),
                DimensionSelect::X => need_insert.insert(ReverseInfo {
                    f: reverse_f.clone(),
                    x: single,
                    y: reverse_y.clone(),
                }),
                DimensionSelect::Y => need_insert.insert(ReverseInfo {
                    f: reverse_f.clone(),
                    x: reverse_x.clone(),
                    y: single,
                }),
            };
        }

        need_delete.insert(target_index);
    }
}
