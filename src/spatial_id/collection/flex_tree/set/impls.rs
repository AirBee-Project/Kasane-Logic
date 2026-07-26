use crate::{FlexId, SpatialIdSet};

impl FromIterator<FlexId> for SpatialIdSet {
    fn from_iter<T: IntoIterator<Item = FlexId>>(iter: T) -> Self {
        let mut set = SpatialIdSet::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl Extend<FlexId> for SpatialIdSet {
    fn extend<T: IntoIterator<Item = FlexId>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}

/// 空間 ID 列から [`SpatialIdSet`] を並列に構築する（`feature = "rayon"`）。
///
/// [`SingleId`](crate::SingleId) / [`RangeId`](crate::RangeId) / [`FlexId`] のいずれの
/// [`SpatialId`](crate::SpatialId) 型でも受け取れる。各要素を [`FlexId`] へ展開してから
/// [`FlexTreeCore::par_build_vec`](crate::FlexTreeCore::par_build_vec) で並列構築する。
/// 集合なので結果は挿入順・チャンク境界に依らず一意（正規形）に定まる。
///
/// ```
/// use kasane_logic::{SingleId, SpatialIdSet};
/// use rayon::prelude::*;
///
/// // 0..1024 は境界整列した連続 X 区間なので 1 異方セル（x_zoom を 10 段浅く）へ畳まれる。
/// let ids: Vec<SingleId> = (0..1024)
///     .map(|x| SingleId::new(20, 0, x, 0).unwrap())
///     .collect();
/// let set: SpatialIdSet = ids.into_par_iter().collect();
/// assert_eq!(set.count(), 1);
/// ```
#[cfg(feature = "rayon")]
impl<S> rayon::iter::FromParallelIterator<S> for SpatialIdSet
where
    S: crate::SpatialId + Send + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = S>,
    {
        use rayon::prelude::*;
        let items: alloc::vec::Vec<(FlexId, ())> = par_iter
            .into_par_iter()
            .flat_map_iter(|s| s.into_iter().map(|f| (f, ())))
            .collect();
        Self {
            inner: crate::FlexTreeCore::par_build_vec(items),
        }
    }
}

/// 既存の [`SpatialIdSet`] へ空間 ID 列を並列にマージする（`feature = "rayon"`）。
#[cfg(feature = "rayon")]
impl<S> rayon::iter::ParallelExtend<S> for SpatialIdSet
where
    S: crate::SpatialId + Send + Sync,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: rayon::iter::IntoParallelIterator<Item = S>,
    {
        use rayon::iter::FromParallelIterator;
        let other = Self::from_par_iter(par_iter);
        self.inner = self.inner.union(&other.inner);
    }
}
