use crate::spatial_id::collection::flex_tree::core::Summary;
use crate::{FlexId, SingleId, SpatialIdTable};

impl<V, S> SpatialIdTable<V, S>
where
    V: PartialEq + Clone,
    S: Summary<V>,
{
    pub fn flex_ids(&self) -> impl Iterator<Item = FlexId> + Clone + '_ {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.flex_ids().flat_map(FlexId::single_ids)
    }
}

/// `(FlexId, V)` 列から [`SpatialIdTable`] を並列に構築する（`feature = "rayon"`）。
///
/// スレッドごとに部分テーブルを組み、和集合で畳む。同じ空間へ異なる値が重なった場合の勝者は
/// `union` の左優先で決まり、逐次 `insert` の後勝ちとは一致しない（値が衝突しない使い方なら結果は一意）。
#[cfg(feature = "rayon")]
impl<V, S> rayon::iter::FromParallelIterator<(FlexId, V)> for SpatialIdTable<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::prelude::*;
        par_iter
            .into_par_iter()
            .fold(Self::default, |mut table, (id, value)| {
                table.insert(id, value);
                table
            })
            .reduce(Self::default, |a, b| Self {
                inner: a.inner.union(&b.inner),
            })
    }
}

/// 既存の [`SpatialIdTable`] へ `(FlexId, V)` 列を並列にマージする（`feature = "rayon"`）。
///
/// 重なる場所は追加する側の値で上書きされる。
#[cfg(feature = "rayon")]
impl<V, S> rayon::iter::ParallelExtend<(FlexId, V)> for SpatialIdTable<V, S>
where
    V: PartialEq + Clone + Send + Sync,
    S: Summary<V> + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::iter::FromParallelIterator;
        let other = Self::from_par_iter(par_iter);
        self.inner = other.inner.union(&self.inner);
    }
}
