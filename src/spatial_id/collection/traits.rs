//! バックエンド非依存のコレクション抽象。
//!
//! [`SpatialIdSet`](crate::SpatialIdSet) / [`SpatialIdTable`](crate::SpatialIdTable) の
//! 実体（FlexTree か Morton か）に依存せず、`expr`（Plan / 二項・単項演算）が共通で
//! 利用する性質を定義する。各バックエンドはこのトレイトを実装する。

use alloc::vec::Vec;

use crate::{ConflictPolicy, FlexId, spatial_id::collection::expr::plan::Plan};

/// コレクションが格納する値型に共通して要求される性質。
pub trait CellValue: Ord + Clone + Send + Sync {}
impl<T: Ord + Clone + Send + Sync> CellValue for T {}

/// 演算の対象となる空間IDコレクションの性質。
#[cfg(feature = "rayon")]
pub trait SpatialIdCollectionBounds: Sized + Sync + Send {}
#[cfg(feature = "rayon")]
impl<T: Sized + Sync + Send> SpatialIdCollectionBounds for T {}

#[cfg(not(feature = "rayon"))]
pub trait SpatialIdCollectionBounds: Sized {}
#[cfg(not(feature = "rayon"))]
impl<T: Sized> SpatialIdCollectionBounds for T {}

pub trait SpatialIdCollection: SpatialIdCollectionBounds {
    /// 各空間IDに紐づく値の型。値を持たない集合では `()`。
    type Value: CellValue;

    /// 空のコレクションを作る（演算結果の組み立て先）。
    fn empty() -> Self;

    /// 1 つの `(FlexId, Value)` を書き込む。
    fn insert(&mut self, key: FlexId, value: Self::Value);

    /// 解決済みセル列から結果コレクションを一括構築する
    /// 全演算子の結果組み立ての共通経路として機能する。
    fn from_cells<I>(cells: I, conflict: &ConflictPolicy<Self::Value>) -> Self
    where
        I: IntoIterator<Item = (FlexId, Self::Value)>,
    {
        if let ConflictPolicy::Overwrite = conflict {
            // 完全一致セルは (最後の出現位置, 最後の値) だけ残す。
            let mut latest: hashbrown::HashMap<FlexId, (usize, Self::Value)> =
                hashbrown::HashMap::new();
            for (pos, (cell, value)) in cells.into_iter().enumerate() {
                latest.insert(cell, (pos, value));
            }
            // 最後の出現順に並べ直してから積む（後勝ちの相対順を保存）。
            let mut ordered: Vec<(usize, FlexId, Self::Value)> = latest
                .into_iter()
                .map(|(id, (pos, value))| (pos, id, value))
                .collect();
            ordered.sort_unstable_by_key(|(pos, _, _)| *pos);

            let mut result = Self::empty();
            for (_, id, value) in ordered {
                result.insert(id, value);
            }
            result
        } else {
            let mut result = Self::empty();
            for (cell, value) in cells {
                let current = result.query(&cell).next().map(|(_, v)| v);
                let resolved = conflict.resolve(current, value);
                result.insert(cell, resolved);
            }
            result
        }
    }

    /// 保持している全ての `(FlexId, Value)` を走査する。
    fn scan(&self) -> impl Iterator<Item = (FlexId, Self::Value)> + '_;

    /// [`scan`](Self::scan) の参照版。値をクローンせず参照で返す（重い値型でのコピー削減）。
    fn scan_ref(&self) -> impl Iterator<Item = (FlexId, &Self::Value)> + '_;

    /// `target` と重なる `(FlexId, Value)` を取得する（2項演算の重なり判定に使う）。
    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, Self::Value)> + 'a;

    /// [`query`](Self::query) の参照版。値をクローンせず参照で返す。
    fn query_ref<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a;

    /// 含まれる要素の最大ズームレベル（正規化・最適化に使う）。
    fn max_zoomlevel(&self) -> Option<u8>;

    /// 空かどうか。
    fn is_empty(&self) -> bool;

    /// このコレクションを起点に、[`Plan`]の組み立てを始める。
    fn plan(self) -> Plan<Self> {
        Plan::Source(self)
    }
}
