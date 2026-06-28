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

    pub fn split_shard(&self, max_flex_id_count: usize) -> alloc::vec::Vec<Self> {
        let mut result = alloc::vec::Vec::new();
        let mut pending = alloc::vec![self.clone()];

        while let Some(piece) = pending.pop() {
            // 閾値以下、または分割不能（1要素以下）ならそのまま確定
            if piece.count() <= max_flex_id_count || piece.count() < 2 {
                result.push(piece);
                continue;
            }

            let Some(region) = piece.inner.balanced_cut() else {
                result.push(piece);
                continue;
            };

            let regions = piece.inner.shard_regions(region);
            let mut rest = piece.clone();
            let mut sub_pieces = alloc::vec::Vec::with_capacity(regions.len());
            for piece_region in regions {
                let extracted: alloc::vec::Vec<(FlexId, V)> = rest.remove(&piece_region).collect();
                let mut sub_piece = Self::new_in_shard(piece_region);
                for (flex_id, value) in extracted {
                    sub_piece.insert(flex_id, value);
                }
                sub_pieces.push(sub_piece);
            }
            pending.extend(sub_pieces);
        }

        result
    }
}
