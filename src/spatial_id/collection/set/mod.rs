use crate::{HyperRect, spatial_id::collection::core::CollectionCore};

///自明なメソットやTrait実装を載せる
pub mod imples;

#[derive(Clone, Debug, Default)]
pub struct SetOnMemory {
    core: CollectionCore<()>,
}

impl SetOnMemory {
    pub fn insert<T: HyperRect>(&mut self, target: T) {}
}
