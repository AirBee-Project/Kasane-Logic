use super::SpatialIdSet;

impl SpatialIdSet {
    /// この集合をシャード分割すべきか判定する。**O(1)**。
    /// 保持する FlexId 数が `max_flex_id_count` を超えていれば `true`。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    /// 互いに素なクリーン領域のシャード列へ分割する。
    /// **O(K·Z²)**・N非依存。各ピースの FlexId 数が `max_flex_id_count` 以下になるまで分割する。
    /// FlexId が `max_flex_id_count` 以下なら自身1つを返す。
    pub fn split_shard(&self, max_flex_id_count: usize) -> alloc::vec::Vec<Self> {
        self.inner
            .split_shard(max_flex_id_count)
            .into_iter()
            .map(|inner| Self { inner })
            .collect()
    }
}
