use crate::{TemporalId, TemporalSet};
use alloc::vec::Vec;
use core::ops::{BitAnd, BitOr, Sub};

impl From<&TemporalId> for TemporalSet {
    fn from(t: &TemporalId) -> Self {
        let mut set = Self::new();
        set.insert(t);
        set
    }
}

impl From<TemporalId> for TemporalSet {
    fn from(t: TemporalId) -> Self {
        let mut set = Self::new();
        set.insert(&t);
        set
    }
}

impl BitOr for &TemporalSet {
    type Output = TemporalSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for &TemporalSet {
    type Output = TemporalSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for &TemporalSet {
    type Output = TemporalSet;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl IntoIterator for &TemporalSet {
    type Item = TemporalId;
    type IntoIter = alloc::vec::IntoIter<TemporalId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}
