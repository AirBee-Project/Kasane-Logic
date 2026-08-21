use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, RangeId};
use alloc::boxed::Box;

/// クエリを実行するためのTrait。読み取りさえできればよい。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    fn read_range_ids(
        &self,
        bounds: &[RangeId],
        token: &CancellationToken,
    ) -> Result<WorkingTree<Self::Value>, Error>;

    /// 木構造を作らずに、フラットな Vec（チャンク）として必要な要素だけを返す
    fn read_range_ids_flat(
        &self,
        bounds: &[RangeId],
        token: &CancellationToken,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, Self::Value)>, Error> {
        // デフォルト実装は WorkingTree 経由（後方互換性）
        let tree = self.read_range_ids(bounds, token)?;
        Ok(tree.into_iter().collect())
    }

    fn read_all(
        self: Box<Self>,
        token: &CancellationToken,
    ) -> Result<WorkingTree<Self::Value>, Error>;

    /// 木構造を作らずに、フラットな Vec（チャンク）として全要素を返す
    fn read_all_flat(
        self: Box<Self>,
        token: &CancellationToken,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, Self::Value)>, Error> {
        // デフォルト実装は WorkingTree 経由（後方互換性）
        let tree = self.read_all(token)?;
        Ok(tree.into_iter().collect())
    }

    fn query(self) -> Query<Self::Value>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::{CancellationToken, RangeId, SingleId, Source, SpatialIdTable};
    use alloc::vec::Vec;

    /// 重なり合う複数 bounds で読んでも、空間IDが重複せず正しい値で返ること。
    ///
    /// 粗いSegment（複数の細かい bounds と交差する）を含めることで、
    /// 同一Segmentが複数回読み出される状況を作っている。
    #[test]
    fn read_range_ids_with_overlapping_bounds_has_no_duplicates() {
        let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
        // z=18 の粗いSegment1つ（z=20 では 4x4 の広がりを持つ）
        table.insert(SingleId::new(18, 0, 100, 100).unwrap(), 7);

        // 上記の粗いSegmentと交差する、細かく分かれた 3 つの領域
        let bounds: Vec<RangeId> = (0..3)
            .map(|i| RangeId::new(20, [0, 0], [400 + i, 400 + i], [400, 400]).unwrap())
            .collect();

        let working = table
            .read_range_ids(&bounds, &CancellationToken::new())
            .unwrap();

        let time_segments: Vec<(crate::FlexId, i32)> = working.into_iter().collect();
        assert_eq!(
            time_segments.len(),
            1,
            "同じSegmentが重複して入っている: {time_segments:?}"
        );
        assert_eq!(time_segments[0].1, 7);
    }

    /// 単一 bounds でも、重なり合う bounds でも同じ結果になること。
    #[test]
    fn overlapping_bounds_match_single_covering_bound() {
        let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
        for x in 400..408u32 {
            table.insert(SingleId::new(20, 0, x, 400).unwrap(), x as i32);
        }

        let single = alloc::vec![RangeId::new(20, [0, 0], [400, 407], [400, 400]).unwrap()];
        // 端が重なる 2 つの領域で同じ範囲を覆う
        let overlapping = alloc::vec![
            RangeId::new(20, [0, 0], [400, 404], [400, 400]).unwrap(),
            RangeId::new(20, [0, 0], [403, 407], [400, 400]).unwrap(),
        ];

        let mut a: Vec<(crate::FlexId, i32)> = table
            .read_range_ids(&single, &CancellationToken::new())
            .unwrap()
            .into_iter()
            .collect();
        let mut b: Vec<(crate::FlexId, i32)> = table
            .read_range_ids(&overlapping, &CancellationToken::new())
            .unwrap()
            .into_iter()
            .collect();
        a.sort();
        b.sort();

        assert_eq!(a, b, "bounds の分割の仕方で結果が変わってはいけない");
        assert_eq!(a.len(), 8);
    }
}
