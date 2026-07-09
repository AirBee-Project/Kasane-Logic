use crate::{ConflictPolicy, TemporalId, TemporalMap};
use alloc::boxed::Box;

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

impl<'a, V: Clone + PartialEq + 'a> IntoIterator for &'a TemporalMap<V> {
    type Item = (TemporalId, &'a V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

impl<V: Clone + PartialEq + 'static> IntoIterator for TemporalMap<V> {
    type Item = (TemporalId, V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter())
    }
}
