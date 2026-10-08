use core::ops::{BitAnd, BitOr, Sub};

use crate::SpatialIdSet;

impl BitOr<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitor(self, rhs: &SpatialIdSet) -> Self::Output {
        SpatialIdSet {
            inner: self.inner.union(&rhs.inner),
            shard: self.shard.filter(|_| self.shard == rhs.shard),
        }
    }
}

impl BitAnd<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitand(self, rhs: &SpatialIdSet) -> Self::Output {
        let shard = match (self.shard, rhs.shard) {
            (Some(a), Some(b)) => a.intersection(&b).or(Some(a)),
            (a, b) => a.or(b),
        };
        SpatialIdSet {
            inner: self.inner.intersection(&rhs.inner),
            shard,
        }
    }
}

impl Sub<&SpatialIdSet> for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn sub(self, rhs: &SpatialIdSet) -> Self::Output {
        SpatialIdSet {
            inner: self.inner.difference(&rhs.inner),
            shard: self.shard,
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
