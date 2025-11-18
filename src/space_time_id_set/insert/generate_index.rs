use crate::space_time_id_set::{Index, SpaceTimeIdSet};

impl SpaceTimeIdSet {
    /// 新しいインデックスを生成する
    ///
    /// IDに割り当てる一意なインデックス番号を生成して返す。
    pub fn generate_index(&mut self) -> Index {
        self.index += 1;
        self.index
    }
}
