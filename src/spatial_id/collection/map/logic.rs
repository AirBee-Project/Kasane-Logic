use crate::BTreeMapTrait;
use crate::spatial_id::ToFlexId;
use crate::spatial_id::collection::map::MapStorage;

pub struct MapLogic<S: MapStorage>(S);

impl<S: MapStorage> MapLogic<S> {
    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub fn insert<I: ToFlexId>(&mut self, target: &I, value: S::Value) {}

    pub fn get<I: ToFlexId>(&mut self, target: &I) -> MapLogic<S> {
        todo!()
    }

    pub fn remove<I: ToFlexId>(&mut self, target: &I) -> MapLogic<S> {
        todo!()
    }
}
