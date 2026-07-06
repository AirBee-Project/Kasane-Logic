use core::ops::{BitAnd, BitOr, Sub};

use crate::SpatialIdSet;
use crate::TemporalSet;
use crate::spatial_id::collection::temporal::{
    SpatioTemporalCore, TSetDifference, TSetIntersection, TSetUnion,
};

impl BitOr<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    /// 和集合。空間が重なる領域では存在時間（[`crate::TemporalSet`]）を union する。
    fn bitor(self, rhs: &SpatialIdSet) -> Self::Output {
        let shard = SpatioTemporalCore::<TemporalSet>::shard_for_union(&self.inner, &rhs.inner);
        SpatialIdSet {
            inner: self.inner.combine_with::<TSetUnion>(&rhs.inner, shard),
        }
    }
}

impl BitAnd<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    /// 積集合。空間が重なり、かつ時間も重なる部分だけが残る。
    fn bitand(self, rhs: &SpatialIdSet) -> Self::Output {
        let shard =
            SpatioTemporalCore::<TemporalSet>::shard_for_intersection(&self.inner, &rhs.inner);
        SpatialIdSet {
            inner: self
                .inner
                .combine_with::<TSetIntersection>(&rhs.inner, shard),
        }
    }
}

impl Sub<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    /// 差集合。空間が重なる領域では時間の差を取り、残った時間だけが残る。
    fn sub(self, rhs: &SpatialIdSet) -> Self::Output {
        let shard = self.inner.shard().cloned();
        SpatialIdSet {
            inner: self.inner.combine_with::<TSetDifference>(&rhs.inner, shard),
        }
    }
}

impl BitOr for SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        &self | &rhs
    }
}

impl BitAnd for SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

impl Sub for SpatialIdSet {
    type Output = SpatialIdSet;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
