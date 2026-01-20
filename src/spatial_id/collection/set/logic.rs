use crate::spatial_id::collection::set::memory::SetOnMemory;
use crate::spatial_id::{ToFlexId, collection::set::SetStorage};
use crate::storage::BTreeMapTrait;

#[derive(Default)]
pub struct SetLogic<S: SetStorage>(S);

impl<S> SetLogic<S>
where
    S: SetStorage + Default,
{
    pub fn open(set_storage: S) -> Self {
        Self(set_storage)
    }

    pub fn close(self) -> S {
        self.0
    }

    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub fn insert<I: ToFlexId>(&mut self, target: &I) {
        todo!()
    }

    pub fn get<I: ToFlexId>(&mut self, target: &I) -> SetOnMemory {
        todo!()
    }

    pub fn remove<I: ToFlexId>(&mut self, target: &I) -> SetOnMemory {
        todo!()
    }

    pub fn union(&self, other: &Self) -> Self {
        todo!()
    }

    pub fn intersection(&self, other: &Self) -> Self {
        todo!()
    }

    pub fn difference(&self, other: &Self) -> Self {
        todo!()
    }
}
