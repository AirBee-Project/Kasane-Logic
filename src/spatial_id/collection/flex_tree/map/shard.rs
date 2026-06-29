use super::SpatialIdMap;
use crate::spatial_id::collection::flex_tree::core::node::Node;
use crate::spatial_id::collection::flex_tree::core::split_child_id;
use crate::{Error, FlexId, Side, SpatialIdError};

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// 保持 [FlexId] 数が `max_flex_id_count` を超えていれば `true`（分割すべき）。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    /// シャードを**下半分／上半分のちょうど2子**へ分割する（親領域を互いに素に完全被覆）。
    /// `((下領域, 下シャード), (上領域, 上シャード))` を返す。シャード領域が未設定なら `None`。
    ///
    /// 過大な子はさらに `split_once` を繰り返すことで二分木状のシャードへ分割される。
    /// 被覆が構造的に保証されるため、空領域への挿入が取りこぼされない。
    pub fn split_once(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let region = self.inner.shard()?.clone();
        let ((lower_region, lower), (upper_region, upper)) =
            self.inner.split_region_binary(&region);
        Some((
            (lower_region, Self { inner: lower }),
            (upper_region, Self { inner: upper }),
        ))
    }

    /// 親領域 `parent_region` の**2兄弟（下半分／上半分）**`a`,`b` を1つのシャードへ統合する
    /// （[`split_once`](Self::split_once) の逆）。境界を跨ぐ同値セルはこの時 compaction される。
    ///
    /// `a`,`b` が `parent_region` の正当な2兄弟でない場合は [`SpatialIdError::InvalidShardMerge`] を返す。
    pub fn merge_sibling_shard(parent_region: FlexId, a: Self, b: Self) -> Result<Self, Error> {
        let level =
            parent_region.f_zoomlevel() + parent_region.x_zoomlevel() + parent_region.y_zoomlevel();
        let axis = Node::<V>::axis(level);
        let expected_lower = split_child_id(&parent_region, axis, Side::Lower);
        let expected_upper = split_child_id(&parent_region, axis, Side::Upper);

        let ra = a.inner.shard();
        let rb = b.inner.shard();
        let ok = (ra == Some(&expected_lower) && rb == Some(&expected_upper))
            || (ra == Some(&expected_upper) && rb == Some(&expected_lower));
        if !ok {
            return Err(Error::SpatialId(SpatialIdError::InvalidShardMerge));
        }

        let mut inner = a.inner.union(&b.inner);
        inner.shard = Some(parent_region);
        Ok(Self { inner })
    }
}
