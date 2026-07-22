use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::traits::WorkingTree;
use crate::{Error, RangeId};

/// クエリの**入力源**。クエリ実行器がデータを読む唯一の口。
///
/// インメモリのコレクション（[`SpatialIdSet`](crate::SpatialIdSet) /
/// [`SpatialIdTable`](crate::SpatialIdTable)）も、ディスク上にシャードされたツリーも、
/// この trait さえ満たせばクエリの入力源になれる。
///
/// 変更系（挿入・削除）は一切要求しない。クエリは分析用であり、入力源を書き換えないため。
pub trait Source: MaybeSendSync {
    /// 演算子が回る作業表現。現状の具象は [`FlexTreeCore`](crate::FlexTreeCore) のみ。
    type Working: WorkingTree;

    /// `bounds` のいずれかに重なるセルだけを読み、作業木を組む（**遅延パス**）。
    ///
    /// [`Query::lazy`] 経由の評価はここしか呼ばない。つまり**範囲読みさえ実装できれば**、
    /// 全件を materialize できないディスク実装でもクエリの入力源になれる。
    fn read_subset(&self, bounds: &[RangeId]) -> Result<Self::Working, Error>;

    /// 全セルを読んで作業木を組む（**eager パス**、[`Query::run`] 用）。
    ///
    /// `Box<Self>` を受けるのは、インメモリ実装が所有権ごと作業木へ移し替えられるようにするため
    /// （クローンを強制しない）。全走査が現実的でない入力源は [`Error::Unsupported`] を返してよく、
    /// その場合は [`Query::lazy`] による領域限定の評価のみを提供する。
    fn read_all(self: Box<Self>) -> Result<Self::Working, Error>;

    /// この入力源を起点にクエリを開始する。
    fn query(self) -> Query<Self::Working>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::{RangeId, SingleId, Source, SpatialIdTable};
    use alloc::vec::Vec;

    /// 重なり合う複数 bounds で読んでも、セルが重複せず正しい値で返ること。
    ///
    /// 粗いセル（複数の細かい bounds と交差する）を含めることで、
    /// 同一セルが複数回読み出される状況を作っている。
    #[test]
    fn read_subset_with_overlapping_bounds_has_no_duplicates() {
        let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
        // z=18 の粗いセル1つ（z=20 では 4x4 の広がりを持つ）
        table.insert(SingleId::new(18, 0, 100, 100).unwrap(), 7);

        // 上記の粗いセルと交差する、細かく分かれた 3 つの領域
        let bounds: Vec<RangeId> = (0..3)
            .map(|i| RangeId::new(20, [0, 0], [400 + i, 400 + i], [400, 400]).unwrap())
            .collect();

        let working = table.read_subset(&bounds).unwrap();

        let cells: Vec<(crate::FlexId, i32)> = working.into_iter().collect();
        assert_eq!(cells.len(), 1, "同じセルが重複して入っている: {cells:?}");
        assert_eq!(cells[0].1, 7);
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

        let mut a: Vec<(crate::FlexId, i32)> =
            table.read_subset(&single).unwrap().into_iter().collect();
        let mut b: Vec<(crate::FlexId, i32)> = table
            .read_subset(&overlapping)
            .unwrap()
            .into_iter()
            .collect();
        a.sort();
        b.sort();

        assert_eq!(a, b, "bounds の分割の仕方で結果が変わってはいけない");
        assert_eq!(a.len(), 8);
    }
}
