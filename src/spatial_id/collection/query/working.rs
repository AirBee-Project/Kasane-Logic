use crate::FlexId;
use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::spatial_id::collection::flex_tree::core::LeavesIntoIter;
use crate::spatial_id::collection::flex_tree::core::SafeValue;

pub struct WorkingTree<V: SafeValue> {
    core: FlexTreeCore<V>,
}

impl<V: SafeValue> WorkingTree<V> {
    /// 空の作業木。
    pub fn new() -> Self {
        Self {
            core: FlexTreeCore::new(),
        }
    }

    /// 保持している[`FlexId`]の数。
    pub fn count(&self) -> usize {
        self.core.count()
    }

    /// Segmentを1つも持たないか。
    pub fn is_empty(&self) -> bool {
        self.core.is_empty()
    }

    pub(crate) fn core(&self) -> &FlexTreeCore<V> {
        &self.core
    }

    pub(crate) fn core_mut(&mut self) -> &mut FlexTreeCore<V> {
        &mut self.core
    }

    pub(crate) fn into_core(self) -> FlexTreeCore<V> {
        self.core
    }

    pub(crate) fn from_core(core: FlexTreeCore<V>) -> Self {
        Self { core }
    }
}

impl<V: SafeValue> Default for WorkingTree<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: SafeValue> Clone for WorkingTree<V> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
        }
    }
}

impl<V: SafeValue> PartialEq for WorkingTree<V> {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl<V: SafeValue> Eq for WorkingTree<V> {}

impl<V: SafeValue> core::fmt::Debug for WorkingTree<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WorkingTree")
            .field("count", &self.count())
            .finish_non_exhaustive()
    }
}

impl<V: SafeValue> FromIterator<(FlexId, V)> for WorkingTree<V> {
    fn from_iter<I: IntoIterator<Item = (FlexId, V)>>(iter: I) -> Self {
        Self {
            core: iter.into_iter().collect(),
        }
    }
}

impl<V: SafeValue> IntoIterator for WorkingTree<V> {
    type Item = (FlexId, V);
    type IntoIter = LeavesIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        self.core.into_iter()
    }
}
