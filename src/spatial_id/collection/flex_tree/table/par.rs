use crate::spatial_id::collection::flex_tree::core::Summary;
use crate::{FlexId, SpatialIdTable};

/// `(FlexId, V)` 列から [`SpatialIdTable`] を並列に構築する（`feature = "rayon"`）。
///
/// 同じ空間へ異なる値が重なった場合にどちらが残るかは、チャンクの分かれ方で決まり、
/// 逐次 `insert` の後勝ちとは一致しない（値が衝突しない使い方なら結果は一意）。
impl<V, S> rayon::iter::FromParallelIterator<(FlexId, V)> for SpatialIdTable<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        Self {
            inner: rayon::iter::FromParallelIterator::from_par_iter(par_iter),
        }
    }
}

/// 既存の [`SpatialIdTable`] へ `(FlexId, V)` 列を並列にマージする（`feature = "rayon"`）。
///
/// 重なる場所は追加する側の値で上書きされる。
impl<V, S> rayon::iter::ParallelExtend<(FlexId, V)> for SpatialIdTable<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        self.inner.par_extend(par_iter);
    }
}
