use crate::{
    BinaryOperator, Error, FlexId, SpatialIdSet, SpatialIdTable, UnaryOperator,
    spatial_id::collection::expr::query::Query,
};

/// `SpatialIdSet`（値を持たない集合）の参照版走査で返す `&()` の実体。
static UNIT: () = ();

/// コレクションが格納する値型に共通して要求される性質。
pub trait CellValue: Ord + Clone + Send + Sync {}
impl<T: Ord + Clone + Send + Sync> CellValue for T {}

#[cfg(not(feature = "rayon"))]
pub trait SpatialIdCollectionBounds: Sized {}
#[cfg(not(feature = "rayon"))]
impl<T: Sized> SpatialIdCollectionBounds for T {}

#[cfg(feature = "rayon")]
pub trait SpatialIdCollectionBounds: Sized + Sync + Send {}
#[cfg(feature = "rayon")]
impl<T: Sized + Sync + Send> SpatialIdCollectionBounds for T {}

pub trait SpatialIdCollection: SpatialIdCollectionBounds {
    type Value: CellValue;
    type Error;

    fn try_insert(&mut self, target: FlexId, value: Self::Value) -> Result<(), Self::Error>;

    fn try_get<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a, Error>;

    fn try_remove<'a>(
        &'a mut self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, Self::Value)> + 'a, Error>;

    fn iter<'a>(&'a self) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a;

    /// [Query]を用いて空間IDの集合を処理します。
    /// 指定した空間IDの集合を直接操作するため、所有権を消費します。
    fn query<U: UnaryOperator, B: BinaryOperator>(self) -> Query<Self, U, B> {
        Query::Source(self)
    }

    // /// [Query]を用いて空間IDの集合を処理します。
    // /// 指定した空間IDの集合を参照して新しい集合を構築するため、元の集合の所有権を消費しません。
    // ///
    // /// 新しい集合の構築は必要部分で行われるため、軽量に動作します。
    // ///
    // /// # Examples
    // ///
    // /// 元の集合を残しつつ、新しい集合を作成する場合:
    // ///
    // /// ```no_run
    // /// use kasane_logic::{SpatialIdTable, SpatialIdCollection};
    // ///
    // /// let table: SpatialIdTable<i32> = SpatialIdTable::new();
    // /// // ... tableに要素を追加 ...
    // ///
    // /// // as_query()は所有権を消費せず、元のtableを維持したまま新しいtableを構築します。
    // /// // 内部では軽量なcloneが行われます。
    // /// let new_table = table.as_query().shift_x(15, 1).run().unwrap();
    // /// ```
    // fn as_query(&self) -> Query<Self> {
    //     (*self).clone().query()
    // }
}

impl SpatialIdCollection for SpatialIdSet {
    type Value = ();
    type Error = Error;

    fn try_insert(&mut self, target: FlexId, _value: Self::Value) -> Result<(), Self::Error> {
        self.insert(target);
        Ok(())
    }

    fn try_get<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a, Error> {
        Ok(self.get(target).map(|id| (id, &UNIT)))
    }

    fn try_remove<'a>(
        &'a mut self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, Self::Value)> + 'a, Error> {
        Ok(self.remove(target).map(|id| (id, ())))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a {
        self.iter().map(|id| (id, &UNIT))
    }
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;
    type Error = Error;

    fn try_insert(&mut self, target: FlexId, value: Self::Value) -> Result<(), Self::Error> {
        self.insert(target, value);
        Ok(())
    }

    fn try_get<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a, Error> {
        Ok(self.get(target))
    }

    fn try_remove<'a>(
        &'a mut self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, Self::Value)> + 'a, Error> {
        Ok(self.remove(target))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a {
        self.iter()
    }
}
