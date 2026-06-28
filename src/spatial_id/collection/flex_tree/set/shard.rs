use super::SpatialIdSet;

impl SpatialIdSet {
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
