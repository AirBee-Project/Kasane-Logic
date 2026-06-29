use super::SpatialIdTable;
use crate::FlexId;

impl<V> SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// シャード領域を返す。`None` が帰ってきた場合はシャードされていない。
    pub fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }
}
