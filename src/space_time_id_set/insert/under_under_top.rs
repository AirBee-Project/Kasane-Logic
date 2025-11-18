use crate::{
    bit_vec::BitVec,
    space_time_id_set::{Index, SpaceTimeIdSet, insert::insert_main_dim::DimensionSelect},
};
/// 分割が必要な次元の情報を保持する構造体
///
/// 各次元について、除外すべきBitVecのリストを保持する。
#[derive(Debug)]
pub struct NeedDivison {
    pub f: Vec<BitVec>,
    pub x: Vec<BitVec>,
    pub y: Vec<BitVec>,
}

impl SpaceTimeIdSet {
    /// 自身を切断する
    ///
    /// あるIndexのIDから特定の次元の特定の部分を除くための情報を記録する。
    /// 後で分割処理を行うために、除外すべきBitVecをdivisionリストに追加する。
    ///
    /// # 引数
    /// * `divison` - 分割情報を格納する構造体
    /// * `target_bit_index` - 対象IDのインデックス
    /// * `target_dim` - 対象次元（F, X, Y のいずれか）
    pub fn under_under_top(
        &self,
        divison: &mut NeedDivison,
        target_bit_index: Index,
        target_dim: DimensionSelect,
    ) {
        println!("under_under_top");

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
