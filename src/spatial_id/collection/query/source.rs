use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::ptr::MaybeSendSync;
use crate::spatial_id::collection::query::execution::Query;
use crate::spatial_id::collection::query::traits::WorkingTree;
use crate::{Error, FlexId, RangeId};

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

/// 複数 `bounds` から読み出したセル列を、作業木の入力として正規化する。
///
/// 単一 bounds なら重複は生じないので何もしない。複数 bounds は領域が重なり得るため、
/// `FlexId` で整列して重複を落とす。[`Source::read_subset`] の実装補助。
pub fn dedup_cells<V>(cells: &mut Vec<(FlexId, V)>, bounds_len: usize) {
    if bounds_len > 1 {
        cells.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        cells.dedup_by(|a, b| a.0 == b.0);
    }
}
