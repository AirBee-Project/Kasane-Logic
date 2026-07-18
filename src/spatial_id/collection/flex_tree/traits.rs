use crate::{
    Error, FlexId, SpatialIdSet, SpatialIdTable, spatial_id::collection::query::execution::Query,
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

    /// `(FlexId, Value)` 列からこのコレクションを構築する。
    ///
    /// FlexTree 実装はマルチコアを活かして並列に構築する（集合は
    /// `FlexTreeCore::par_build_vec`）。
    /// `map_rebuild` の再構築段はこのメソッドに委ねられる。
    fn from_items(items: alloc::vec::Vec<(FlexId, Self::Value)>) -> Self;

    /// 各格納要素 `(FlexId, &Value)` を関数 `f` で 0 個以上の `(FlexId, Value)` へ写し、
    /// 写した全体からコレクションを再構築して返す。
    ///
    /// `shift` のような「全要素を独立に写して作り直す」単項演算の共通土台。各要素の写像は
    /// 互いに独立なので、`rayon` 有効時は写像段（`f` の適用）と再構築段（`from_items`）の
    /// 双方が並列化され、FlexTree のマルチコア実装を最大限に活かす。
    ///
    /// `f` はエラーを返しうる（範囲外シフト等）。1 要素でもエラーになれば全体を中断して返す。
    ///
    /// # 値の衝突
    /// 写像先が重なる場合、再構築は `from_items` の合成規則に従う（集合は和、テーブルは
    /// 後勝ち）。`shift` は平行移動で単射のため衝突しない。
    fn map_rebuild<F, I>(&self, f: F) -> Result<Self, Error>
    where
        Self: Sized,
        F: Fn(FlexId, &Self::Value) -> Result<I, Error> + Send + Sync,
        I: IntoIterator<Item = (FlexId, Self::Value)>,
    {
        let snapshot: alloc::vec::Vec<(FlexId, Self::Value)> =
            self.iter().map(|(id, v)| (id, v.clone())).collect();

        #[cfg(feature = "rayon")]
        let mapped: alloc::vec::Vec<(FlexId, Self::Value)> = {
            use rayon::prelude::*;
            snapshot
                .into_par_iter()
                .map(|(id, v)| f(id, &v).map(|it| it.into_iter().collect::<alloc::vec::Vec<_>>()))
                .collect::<Result<alloc::vec::Vec<_>, Error>>()?
                .into_iter()
                .flatten()
                .collect()
        };
        #[cfg(not(feature = "rayon"))]
        let mapped: alloc::vec::Vec<(FlexId, Self::Value)> = {
            let mut out = alloc::vec::Vec::new();
            for (id, v) in snapshot {
                out.extend(f(id, &v)?);
            }
            out
        };

        Ok(Self::from_items(mapped))
    }

    /// [Query]を用いて空間IDの集合を処理します。
    /// 指定した空間IDの集合を直接操作するため、所有権を消費します。
    /// コレクションに対するクエリを開始する
    fn query(self) -> Query<Self> {
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

    fn from_items(items: alloc::vec::Vec<(FlexId, Self::Value)>) -> Self {
        // 集合は FlexId のみを並列構築する（値は ()）。rayon 有効時は
        // `FromParallelIterator`（内部で `par_build_vec`）が並列に組み立てる。
        #[cfg(feature = "rayon")]
        {
            use rayon::prelude::*;
            items.into_par_iter().map(|(id, _)| id).collect()
        }
        #[cfg(not(feature = "rayon"))]
        {
            items.into_iter().map(|(id, _)| id).collect()
        }
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

    fn from_items(items: alloc::vec::Vec<(FlexId, Self::Value)>) -> Self {
        // テーブルは値の辞書索引を持つため、現状は逐次挿入で構築する
        // （値の重複排除を伴う並列構築は今後の課題）。
        let mut table = SpatialIdTable::new();
        for (id, value) in items {
            table.insert(id, value);
        }
        table
    }
}
