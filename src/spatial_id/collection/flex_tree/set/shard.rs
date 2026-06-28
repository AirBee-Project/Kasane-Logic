use crate::FlexId;

use super::SpatialIdSet;

impl SpatialIdSet {
    /// シャード領域を返す。`None` が帰ってきた場合はシャードされていない。
    pub fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }

    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    pub fn split_shard(&self, max_flex_id_count: usize) -> alloc::vec::Vec<Self> {
        self.inner
            .split_shard(max_flex_id_count)
            .into_iter()
            .map(|inner| Self { inner })
            .collect()
    }
}
