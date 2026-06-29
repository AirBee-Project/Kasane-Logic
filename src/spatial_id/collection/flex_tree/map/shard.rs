use super::SpatialIdMap;
use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::{Error, FlexId, SpatialIdError};
use alloc::vec::Vec;

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
    /// 過大な子はさらに `split_shard` を繰り返すことで二分木状のシャードへ分割される。
    /// 被覆が構造的に保証されるため、空領域への挿入が取りこぼされない。なお分割は常に
    /// 正準軸（level % 3 で F→X→Y）で行うが、片側が空になる退化分割の畳み込み（パス圧縮）は
    /// 上位（シャード層）の責務とする。これにより内部木は正準構造のまま、materialize される
    /// ポインタノードはデータが実際に割れる軸（例: XY）で枝分かれする。
    pub fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let ((lower_region, lower), (upper_region, upper)) = self.inner.split_shard()?;
        Some((
            (lower_region, Self { inner: lower }),
            (upper_region, Self { inner: upper }),
        ))
    }

    /// `parent_region` を被覆する子シャード群 `children` を1つのシャードへ統合する
    /// （[`split_shard`](Self::split_shard) を繰り返した分割の逆）。境界を跨ぐ同値セルはこの時
    /// compaction される。パス圧縮で可変数（2 以上）になり得るポインタノードのサブツリー畳み込みに使う。
    ///
    /// 各 `children` のシャード領域が `parent_region` に含まれていない場合は
    /// [`SpatialIdError::InvalidShardMerge`] を返す（被覆そのものは呼び出し側の分割不変条件が保証する）。
    pub fn merge_shards(parent_region: FlexId, children: Vec<Self>) -> Result<Self, Error> {
        // 各子は親領域に内包されていなければならない（region ⊆ parent_region）。
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
