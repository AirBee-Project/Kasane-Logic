use crate::{
    ConflictPolicy, FlexId, SpatialIdSet, SpatialIdTable, spatial_id::collection::expr::plan::Plan,
};

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

    /// 解決済みセル列から結果コレクションを一括構築する（全演算子の結果組み立ての共通経路）。
    ///
    /// 既定は逐次 `insert`。`conflict` が `Overwrite` 以外のときは書き込み前に現状を
    /// [`query`](Self::query) して解決するため**入力順に依存する**ので、呼び出し側は順序を
    /// 保つこと。値インデックス等の付随構造を持つ実装は、これをオーバーライドして一括構築へ
    /// 差し替えてよい。
    fn from_cells<I>(cells: I, conflict: &ConflictPolicy<Self::Value>) -> Self
    where
        I: IntoIterator<Item = (FlexId, Self::Value)>,
    {
        let mut result = Self::empty();
        for (cell, value) in cells {
            let resolved = if let ConflictPolicy::Overwrite = conflict {
                value
            } else {
                let current = result.query(&cell).next().map(|(_, v)| v);
                conflict.resolve(current, value)
            };
            result.insert(cell, resolved);
        }
        result
    }

    /// 保持している全ての `(FlexId, Value)` を走査する。
    fn scan(&self) -> impl Iterator<Item = (FlexId, Self::Value)> + '_;

    /// `target` と重なる `(FlexId, Value)` を取得する（2項演算の重なり判定に使う）。
    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, Self::Value)> + 'a;

    /// 含まれる要素の最大ズームレベル（正規化・最適化に使う）。
    fn max_zoomlevel(&self) -> Option<u8>;

    /// 空かどうか。
    fn is_empty(&self) -> bool;

    /// このコレクションを起点に、[`Plan`]の組み立てを始める。
    fn plan(self) -> Plan<Self> {
        Plan::Source(self)
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

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, V)> + 'a {
        self.get(target).map(|(id, v)| (id, v.clone()))
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

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, ())> + 'a {
        self.get(target).map(|id| (id, ()))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdSet::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdSet::is_empty(self)
    }
}
