use std::collections::BTreeMap;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, SpaceTimeIdSet},
};
impl SpaceTimeIdSet {
    /// 与えられた次元において、下位の範囲の個数を読み取る
    ///
    /// 指定されたBitVecの下に存在するIDの数を返す。
    /// これは挿入時の最適な次元を選択するために使用される。
    ///
    /// # 引数
    /// * `btree` - 検索対象のBTreeMap
    /// * `encoded` - 対象のBitVec
    pub fn search_under_count(btree: &BTreeMap<BitVec, LayerInfo>, encoded: &BitVec) -> usize {
        match btree.get(encoded) {
            Some(v) => v.count,
            None => 0,
        }
    }
}
