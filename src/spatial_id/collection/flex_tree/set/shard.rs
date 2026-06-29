use alloc::vec::Vec;

use super::SpatialIdSet;
use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::{Error, FlexId, SpatialIdError};

impl SpatialIdSet {
    /// シャード領域を返す。`None` が帰ってきた場合はシャードされていない。
    pub fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }

    /// 保持 [FlexId] 数が `max_flex_id_count` を超えていれば `true`（分割すべき）。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.inner.should_split_shard(max_flex_id_count)
    }

    /// シャードを**下半分／上半分のちょうど2子**へ分割する（親領域を互いに素に完全被覆）。
    /// `((下領域, 下シャード), (上領域, 上シャード))` を返す。シャード領域が未設定なら `None`。
    ///
    /// [`SpatialIdMap::split_shard`](crate::SpatialIdMap::split_shard) と対称。退化分割の畳み込み
    /// （パス圧縮）は上位（シャード層）の責務で、本メソッドは常に正準軸で1段だけ分割する。
    pub fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let ((lower_region, lower), (upper_region, upper)) = self.inner.split_shard()?;
        Some((
            (lower_region, Self { inner: lower }),
            (upper_region, Self { inner: upper }),
        ))
    }

    /// `parent_region` を被覆する子シャード群 `children` を1つのシャードへ統合する
    /// （[`split_shard`](Self::split_shard) を繰り返した分割の逆）。
    ///
    /// 各 `children` のシャード領域が `parent_region` に含まれていない場合は
    /// [`SpatialIdError::InvalidShardMerge`] を返す（被覆そのものは呼び出し側の分割不変条件が保証する）。
    pub fn merge_shards(parent_region: FlexId, children: Vec<Self>) -> Result<Self, Error> {
        for c in &children {
            if let Some(r) = c.inner.shard()
                && parent_region.intersection(r).as_ref() != Some(r)
            {
                return Err(Error::SpatialId(SpatialIdError::InvalidShardMerge));
            }
        }

        let mut inner = FlexTreeCore::new_in_shard(parent_region.clone());
        for c in children {
            inner = inner.union(&c.inner);
        }
        inner.shard = Some(parent_region);
        Ok(Self { inner })
    }
}
