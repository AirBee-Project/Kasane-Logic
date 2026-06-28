use super::SpatialIdMap;
use crate::{FlexId, FlexTreeCore};

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
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

    /// 親領域 `parent_region` を互いに素に覆う**兄弟シャード群を1つのシャードへ統合**する。
    ///
    /// [`split_shard`](Self::split_shard) の逆操作。`children` は split で生じた、
    /// 親領域の互いに素な被覆である前提（各 child は自身の領域に閉じている）。
    /// 結合は `union` で行い、境界を跨いで隣接する同値セルはこの時点で compaction される。
    /// 結果のシャード領域は `parent_region` に設定される。
    pub fn merge_siblings(parent_region: FlexId, children: impl IntoIterator<Item = Self>) -> Self {
        let mut acc: Option<FlexTreeCore<V>> = None;
        for child in children {
            acc = Some(match acc {
                None => child.inner,
                Some(a) => a.union(&child.inner),
            });
        }
        let mut inner = acc.unwrap_or_else(|| FlexTreeCore::new_in_shard(parent_region.clone()));
        inner.shard = Some(parent_region);
        Self { inner }
    }
}
