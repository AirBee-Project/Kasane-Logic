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

pub trait SpatialIdCollection:
    SpatialIdCollectionBounds + Clone + Default + IntoIterator<Item = (FlexId, Self::Value)>
{
    /// 各空間IDに紐づく値の型。値を持たない集合では `()`。
    type Value: CellValue;

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

            let mut result = Self::default();
            for (_, id, value) in ordered {
                result.insert(id, value);
            }
            result
        } else {
            let mut result = Self::default();
            for (cell, value) in cells {
                let current = result.get(&cell).next().map(|(_, v)| v);
                let resolved = conflict.resolve(current, value);
                result.insert(cell, resolved);
            }
            result
        }
    }

    /// コレクションの要素を参照で走査する。
    fn iter(&self) -> impl Iterator<Item = (FlexId, &Self::Value)> + '_;

    /// `target` と重なる `(FlexId, Value)` を取得する（2項演算の重なり判定に使う）。
    fn get<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, Self::Value)> + 'a;

    /// [`query`](Self::query) の参照版。値をクローンせず参照で返す。
    fn get_ref<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a;

    /// 含まれる要素の最大ズームレベル（正規化・最適化に使う）。
    fn max_zoomlevel(&self) -> Option<u8>;

    /// 空かどうか。
    fn is_empty(&self) -> bool;

    /// [Query]を用いて空間IDの集合を処理します。
    /// 指定した空間IDの集合を直接操作するため、所有権を消費します。
    ///
    /// # Examples
    ///
    /// 元のコレクションを直接書き換える場合:
    ///
    /// ```rust
    /// use kasane_logic::{SpatialIdTable, SpatialIdCollection};
    ///
    /// let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
    /// // ... tableに要素を追加 ...
    ///
    /// // query()は所有権を消費し、メモリを再利用して高速に元のtableを直接書き換えます。
    /// table = table.query().shift_x(15, 1).run().unwrap();
    /// ```
    fn query(self) -> Query<Self> {
        Query::Source(self)
    }

    /// [Query]を用いて空間IDの集合を処理します。
    /// 指定した空間IDの集合を参照して新しい集合を構築するため、元の集合の所有権を消費しません。
    ///
    /// 新しい集合の構築は必要部分で行われるため、軽量に動作します。
    ///
    /// # Examples
    ///
    /// 元の集合を残しつつ、新しい集合を作成する場合:
    ///
    /// ```rust
    /// use kasane_logic::{SpatialIdTable, SpatialIdCollection};
    ///
    /// let table: SpatialIdTable<i32> = SpatialIdTable::new();
    /// // ... tableに要素を追加 ...
    ///
    /// // as_query()は所有権を消費せず、元のtableを維持したまま新しいtableを構築します。
    /// // 内部では軽量なcloneが行われます。
    /// let new_table = table.as_query().shift_x(15, 1).run().unwrap();
    /// ```
    fn as_query(&self) -> Query<Self> {
        (*self).clone().query()
    }

    /// コレクション内のすべての値を更新します。
    fn map_values_in_place<F>(&mut self, f: F)
    where
        F: FnMut(&mut Self::Value);
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;

    fn insert(&mut self, key: FlexId, value: V) {
        SpatialIdTable::insert(self, key, value);
    }

    fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.iter()
    }

    fn get<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, V)> + 'a {
        self.get(target).map(|(id, v)| (id, v.clone()))
    }

    fn get_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.get(target)
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdTable::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdTable::is_empty(self)
    }

    fn map_values_in_place<F>(&mut self, f: F)
    where
        F: FnMut(&mut Self::Value),
    {
        SpatialIdTable::map_values_in_place(self, f);
    }
}

impl SpatialIdCollection for SpatialIdSet {
    type Value = ();

    fn insert(&mut self, key: FlexId, _value: ()) {
        SpatialIdSet::insert(self, key);
    }

    fn iter(&self) -> impl Iterator<Item = (FlexId, &())> + '_ {
        SpatialIdSet::iter(self).map(|id| (id, &UNIT))
    }

    fn get<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, ())> + 'a {
        self.get(target).map(|id| (id, ()))
    }

    fn get_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a ())> + 'a {
        self.get(target).map(|id| (id, &UNIT))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdSet::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdSet::is_empty(self)
    }

    fn map_values_in_place<F>(&mut self, _f: F)
    where
        F: FnMut(&mut Self::Value),
    {
        // SpatialIdSet's value is (), so no mutation is needed.
    }
}
