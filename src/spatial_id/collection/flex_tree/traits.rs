use alloc::vec::Vec;

use crate::{
    ConflictPolicy, FlexId, SpatialIdSet, SpatialIdTable,
    spatial_id::collection::expr::query::Query,
};

/// `SpatialIdSet`（値を持たない集合）の参照版走査で返す `&()` の実体。
static UNIT: () = ();

/// コレクションが格納する値型に共通して要求される性質。
pub trait CellValue: Ord + Clone + Send + Sync {}
impl<T: Ord + Clone + Send + Sync> CellValue for T {}

/// 演算の対象となる空間IDコレクションの性質。
///
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

    /// このコレクションを起点に、[`Query`]の組み立てを始める。
    fn into_query(self) -> Query<Self> {
        Query::Source(self)
    }
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;

    fn empty() -> Self {
        SpatialIdTable::new()
    }

    fn insert(&mut self, key: FlexId, value: V) {
        SpatialIdTable::insert(self, key, value);
    }

    fn scan(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        self.iter().map(|(id, v)| (id, v.clone()))
    }

    fn scan_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.iter()
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, V)> + 'a {
        self.get(target).map(|(id, v)| (id, v.clone()))
    }

    fn query_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.get(target)
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdTable::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdTable::is_empty(self)
    }
}

impl SpatialIdCollection for SpatialIdSet {
    type Value = ();

    fn empty() -> Self {
        SpatialIdSet::new()
    }

    fn insert(&mut self, key: FlexId, _value: ()) {
        SpatialIdSet::insert(self, key);
    }

    fn scan(&self) -> impl Iterator<Item = (FlexId, ())> + '_ {
        self.iter().map(|id| (id, ()))
    }

    fn scan_ref(&self) -> impl Iterator<Item = (FlexId, &())> + '_ {
        self.iter().map(|id| (id, &UNIT))
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, ())> + 'a {
        self.get(target).map(|id| (id, ()))
    }

    fn query_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a ())> + 'a {
        self.get(target).map(|id| (id, &UNIT))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdSet::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdSet::is_empty(self)
    }
}
