use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::{FlexId, SingleId, SpatialIdMap};

impl<V> SpatialIdMap<V>
where
    V: SafeValue,
{
    pub fn flex_ids(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.inner.single_ids()
    }
}

impl<V> FromIterator<(FlexId, V)> for SpatialIdMap<V>
where
    V: SafeValue,
{
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        let mut map = SpatialIdMap::new();
        for (id, value) in iter {
            map.insert(id, value);
        }
        map
    }
}

impl<V> Extend<(FlexId, V)> for SpatialIdMap<V>
where
    V: SafeValue,
{
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        for (id, value) in iter {
            self.insert(id, value);
        }
    }
}

/// `(FlexId, V)` 列から [`SpatialIdMap`] を並列に構築する（`feature = "rayon"`）。
///
/// [`FlexTreeCore::par_build_vec`](crate::FlexTreeCore::par_build_vec) で並列構築する。
/// 同じ空間へ異なる値が重なった場合の解決は `union` と同じ左優先で、逐次 `insert` の
/// 後勝ちとは一致しない（値が衝突しない使い方なら結果は一意）。
#[cfg(feature = "rayon")]
impl<V> rayon::iter::FromParallelIterator<(FlexId, V)> for SpatialIdMap<V>
where
    V: SafeValue,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::iter::ParallelIterator;
        let items: alloc::vec::Vec<(FlexId, V)> = par_iter.into_par_iter().collect();
        Self {
            inner: crate::FlexTreeCore::par_build_vec(items),
        }
    }
}

/// 既存の [`SpatialIdMap`] へ `(FlexId, V)` 列を並列にマージする（`feature = "rayon"`）。
#[cfg(feature = "rayon")]
impl<V> rayon::iter::ParallelExtend<(FlexId, V)> for SpatialIdMap<V>
where
    V: SafeValue,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = (FlexId, V)>,
    {
        use rayon::iter::FromParallelIterator;
        let other = Self::from_par_iter(par_iter);
        self.inner = self.inner.union(&other.inner);
    }
}
