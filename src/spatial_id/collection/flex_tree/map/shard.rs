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
    /// このシャード（[`shard`](Self::shard) 領域）を、現在のrootの軸で2分割し、切り取った部分木を `((下のシャード領域, 下の実体), (上のシャード領域, 上の実体))` で返す。
    /// シャード領域が未設定なら `None`を返す。
    pub fn split_shard(&self) -> Option<((FlexId, Self), (FlexId, Self))> {
        let ((lower_region, lower), (upper_region, upper)) = self.inner.split_shard()?;
        Some((
            (lower_region, Self { inner: lower }),
            (upper_region, Self { inner: upper }),
        ))
    }

    /// シャードされている複数の[SpatialIdMap]を、`parent_region` に閉じた1つの[SpatialIdMap]へ統合する。
    ///
    /// 次のいずれかに該当すると [`SpatialIdError::InvalidShardMerge`] を返す：
    /// - シャード領域が未設定（`None`）の子が含まれる。
    /// - 子のシャード領域が `parent_region` からはみ出している。
    /// - 子同士のシャード領域が重なっている。
    pub fn merge_shards(parent_region: FlexId, children: Vec<Self>) -> Result<Self, Error> {
        // はみ出していないか
        let mut regions: Vec<FlexId> = Vec::with_capacity(children.len());
        for c in &children {
            let r = c
                .inner
                .shard()
                .ok_or(Error::SpatialId(SpatialIdError::InvalidShardMerge))?
                .clone();
            if parent_region.intersection(&r).as_ref() != Some(&r) {
                return Err(Error::SpatialId(SpatialIdError::InvalidShardMerge));
            }
            regions.push(r);
        }

        // 子同士は互いに素であること
        for i in 0..regions.len() {
            for j in (i + 1)..regions.len() {
                if regions[i].intersection(&regions[j]).is_some() {
                    return Err(Error::SpatialId(SpatialIdError::InvalidShardMerge));
                }
            }
        }

        let mut inner = FlexTreeCore::new_in_shard(parent_region.clone());
        for c in children {
            inner = inner
                .combine_with::<crate::spatial_id::collection::flex_tree::core::node_ops::TMapOverwrite>(
                    &c.inner,
                    Some(parent_region.clone()),
                );
        }
        Ok(Self { inner })
    }
}
