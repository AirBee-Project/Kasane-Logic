use crate::{FlexId, FlexTreeCore, IterFlexIds, SingleId};
pub mod convert;
pub mod ops;
pub mod test;

#[derive(Default, Clone)]
pub struct FlexTreeSet {
    inner: FlexTreeCore<()>,
}

impl FlexTreeSet {
    pub fn new() -> Self {
        FlexTreeSet::default()
    }

    pub fn insert<S: IterFlexIds>(&mut self, target: S) {
        self.inner.insert(target, ());
    }

    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = FlexId> + 'a
    where
        S: IterFlexIds,
    {
        self.inner.get(target).map(move |(flex_id, _value)| flex_id)
    }

    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> impl Iterator<Item = FlexId> {
        self.inner
            .remove(target)
            .map(move |(flex_id, _value)| flex_id)
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }

    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    pub fn flat_single_ids(&self) -> std::vec::IntoIter<SingleId> {
        self.inner
            .flat_single_ids()
            .map(|(single_id, _)| single_id)
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = FlexId> {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }
}
