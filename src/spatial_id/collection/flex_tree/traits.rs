use crate::{
    Error, FlexId, FlexTreeCore, SpatialIdSet, SpatialIdTable,
    spatial_id::collection::query::{execution::Query, traits::WorkingTree},
};

/// `SpatialIdSet`（値を持たない集合）の参照版走査で返す `&()` の実体。
static UNIT: () = ();

/// Table の入口/出口変換（`try_into_working`/`try_from_working`）で、これ未満なら rayon を使わず逐次で組む閾値。
/// 単発・小規模クエリで rayon 起動コスト（par_build / from_par_iter の par_sort 等）を避ける。
#[cfg(feature = "rayon")]
const SEQ_CONVERT_THRESHOLD: usize = 512;

#[cfg(not(feature = "rayon"))]
pub trait CellValue: Ord + Clone {}
#[cfg(not(feature = "rayon"))]
impl<T: Ord + Clone> CellValue for T {}

#[cfg(feature = "rayon")]
pub trait CellValue: Ord + Clone + Send + Sync {}
#[cfg(feature = "rayon")]
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

    /// クエリ実行器が演算子を回すための作業表現。具象型（[`FlexTreeCore`] 等）はここに閉じ込め、
    /// `SpatialIdCollection` の公開シグネチャには現れない。実装は [`WorkingTree`] を満たす必要がある
    /// （`map_rebuild`/`map_rebuild_with` など、演算子が実際に使うメソッドのみを持つ境界）。
    type Working: WorkingTree<Value = Self::Value>;

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

    /// クエリ実行の**入口変換**: 所有権ごと作業表現 [`Self::Working`](Self::Working) へ写す。
    ///
    /// 実行器は連鎖の入口で 1 回だけこれを呼び、以降の全演算子を `Self::Working` 上で回す。Table は
    /// rank ツリーを辞書で実体値へ展開する（演算子ごとの再 intern を無くすための境界）。他の
    /// `try_*` と同じく `Result` で包む（ディスク実装等、失敗しうる変換を将来許容するため）。
    fn try_into_working(self) -> Result<Self::Working, Error>;

    /// クエリ実行の**出口変換**: 作業表現 [`Self::Working`](Self::Working) からコレクションを組む。
    ///
    /// 実行器は連鎖の出口で 1 回だけ呼ぶ。Table は実体値を辞書へ intern し直す。
    fn try_from_working(working: Self::Working) -> Result<Self, Error>;

    /// [Query]を用いて空間IDの集合を処理します。
    /// 指定した空間IDの集合を直接操作するため、所有権を消費します。
    /// コレクションに対するクエリを開始する
    fn query(self) -> Query<Self> {
        Query::Source(self)
    }
}

impl SpatialIdCollection for SpatialIdSet {
    type Value = ();
    type Working = FlexTreeCore<()>;

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

    fn try_into_working(self) -> Result<FlexTreeCore<()>, Error> {
        Ok(SpatialIdSet::into_core(self))
    }

    fn try_from_working(working: FlexTreeCore<()>) -> Result<Self, Error> {
        Ok(SpatialIdSet::from_core(working))
    }
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;
    type Working = FlexTreeCore<V>;

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

    fn try_into_working(self) -> Result<FlexTreeCore<V>, Error> {
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
                Ok(core)
            } else {
                Ok(FlexTreeCore::par_build_vec(items))
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            let mut core = FlexTreeCore::new();
            for (id, value) in self {
                core.insert(id, value);
            }
            Ok(core)
        }
    }

    fn try_from_working(core: FlexTreeCore<V>) -> Result<Self, Error> {
        // 実体値の互いに素なセルを辞書へ intern し直す。小入力は逐次で（rayon 起動コスト回避）。
        #[cfg(feature = "rayon")]
        {
            let cells: alloc::vec::Vec<(FlexId, V)> = core.into_iter().collect();
            use rayon::iter::FromParallelIterator;
            if cells.len() < SEQ_CONVERT_THRESHOLD {
                Ok(cells.into_iter().collect())
            } else {
                Ok(SpatialIdTable::from_par_iter(cells))
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            Ok(core.into_iter().collect())
        }
    }
}
