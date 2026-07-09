use core::ops::Sub;

use crate::{ConflictPolicy, TemporalId, TemporalMap};

impl<V: Clone + Ord> TemporalMap<V> {
    /// 和（both は `policy` で値解決、片側はそのまま）。
    pub fn union(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.union(&other.0, policy))
    }

    /// 積（both のみ・`policy` で値解決）。
    pub fn intersection(&self, other: &Self, policy: &ConflictPolicy<V>) -> Self {
        Self(self.0.sweep(&other.0, |a, b| match (a, b) {
            (Some(a), Some(b)) => Some(policy.resolve(Some(a.clone()), b.clone())),
            _ => None,
        }))
    }
}

impl<V: Clone + PartialEq> Sub for &TemporalMap<V> {
    type Output = TemporalMap<V>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl<V: Clone + PartialEq> IntoIterator for &TemporalMap<V> {
    type Item = (TemporalId, V);
    type IntoIter = alloc::vec::IntoIter<(TemporalId, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
            .map(|(t, v)| (t, v.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
