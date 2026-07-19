use crate::{
    Error, FlexId, FlexTreeCore, SpatialIdSet, SpatialIdTable,
    spatial_id::collection::query::execution::Query,
};

/// `SpatialIdSet`（値を持たない集合）の参照版走査で返す `&()` の実体。
static UNIT: () = ();

/// Table の入口/出口変換（`into_core`/`from_core`）で、これ未満なら rayon を使わず逐次で組む閾値。
/// 単発・小規模クエリで rayon 起動コスト（par_build / from_par_iter の par_sort 等）を避ける。
#[cfg(feature = "rayon")]
const SEQ_CONVERT_THRESHOLD: usize = 512;

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

    fn try_insert(&mut self, target: FlexId, value: Self::Value) -> Result<(), Error>;

    fn try_get<'a>(
        &'a self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a, Error>;

    fn try_remove<'a>(
        &'a mut self,
        target: &'a FlexId,
    ) -> Result<impl Iterator<Item = (FlexId, Self::Value)> + 'a, Error>;

    fn iter<'a>(&'a self) -> impl Iterator<Item = (FlexId, &'a Self::Value)> + 'a;

    /// クエリ実行の**入口変換**: 所有権ごと実体値の [`FlexTreeCore`] へ写す。
    ///
    /// 実行器は連鎖の入口で 1 回だけこれを呼び、以降の全演算子を `FlexTreeCore<Self::Value>` 上で
    /// 回す。Table は rank ツリーを辞書で実体値へ展開する（演算子ごとの再 intern を無くすための境界）。
    fn into_core(self) -> FlexTreeCore<Self::Value>;

    /// クエリ実行の**出口変換**: 実体値の [`FlexTreeCore`] からコレクションを組む。
    ///
    /// 実行器は連鎖の出口で 1 回だけ呼ぶ。Table は実体値を辞書へ intern し直す。
    fn from_core(core: FlexTreeCore<Self::Value>) -> Self;

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

    fn try_insert(&mut self, target: FlexId, _value: Self::Value) -> Result<(), Error> {
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

    fn into_core(self) -> FlexTreeCore<()> {
        SpatialIdSet::into_core(self)
    }

    fn from_core(core: FlexTreeCore<()>) -> Self {
        SpatialIdSet::from_core(core)
    }
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;

    fn try_insert(&mut self, target: FlexId, value: Self::Value) -> Result<(), Error> {
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

    fn into_core(self) -> FlexTreeCore<V> {
        // rank ツリーを辞書で実体値へ展開。Table のセルは互いに素なので union（par_build_vec）で正しい。
        #[cfg(feature = "rayon")]
        {
            let items: alloc::vec::Vec<(FlexId, V)> = self.into_iter().collect();
            // 小入力では rayon 起動コストが利得を上回るので逐次挿入で組む（単発クエリの入口変換の固定費を削る）。
            if items.len() < SEQ_CONVERT_THRESHOLD {
                let mut core = FlexTreeCore::new();
                for (id, value) in items {
                    core.insert(id, value);
                }
                core
            } else {
                FlexTreeCore::par_build_vec(items)
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            let mut core = FlexTreeCore::new();
            for (id, value) in self {
                core.insert(id, value);
            }
            core
        }
    }

    fn from_core(core: FlexTreeCore<V>) -> Self {
        // 実体値の互いに素なセルを辞書へ intern し直す。小入力は逐次で（rayon 起動コスト回避）。
        #[cfg(feature = "rayon")]
        {
            let cells: alloc::vec::Vec<(FlexId, V)> = core.into_iter().collect();
            use rayon::iter::FromParallelIterator;
            if cells.len() < SEQ_CONVERT_THRESHOLD {
                cells.into_iter().collect()
            } else {
                SpatialIdTable::from_par_iter(cells)
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            core.into_iter().collect()
        }
    }
}
