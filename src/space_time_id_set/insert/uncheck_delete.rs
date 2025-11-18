use crate::space_time_id_set::{Index, SpaceTimeIdSet};

impl SpaceTimeIdSet {
    /// 検証なしでIDをセットから削除する
    ///
    /// 指定されたインデックスのIDを各次元から削除する。
    ///
    /// # 引数
    /// * `index` - 削除するIDのインデックス
    pub fn uncheck_delete(&mut self, index: &Index) {
        let removed = self.reverse.remove(index).unwrap();

        self.f.remove(&removed.f);
        self.x.remove(&removed.x);
        self.y.remove(&removed.y);
    }
}
