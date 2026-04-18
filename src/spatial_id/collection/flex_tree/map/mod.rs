use crate::{FlexId, FlexTreeCore, IterFlexIds, SingleId};
pub mod convert;

#[derive(Default, Clone)]
pub struct FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    inner: FlexTreeCore<V>,
}

impl<V> FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self {
            inner: FlexTreeCore::new(),
        }
    }

    pub fn insert<S: IterFlexIds>(&mut self, target: S, value: V) {
        self.inner.insert(target, value);
    }

    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds,
    {
        self.inner.get(target)
    }

    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove(target)
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, V)> {
        self.inner.flat_single_ids()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.iter()
    }
}
