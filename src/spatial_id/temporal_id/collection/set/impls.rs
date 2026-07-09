use crate::{TemporalId, TemporalSet};
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

impl<'a> IntoIterator for &'a TemporalSet {
    type Item = TemporalId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.iter().map(|(t, ())| t))
    }
}

impl IntoIterator for TemporalSet {
    type Item = TemporalId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter().map(|(t, ())| t))
    }
}
