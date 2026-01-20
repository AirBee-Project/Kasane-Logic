use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::{
    spatial_id::{collection::Rank, flex_id::FlexId, segment::Segment},
    storage::BTreeMapTrait,
};

pub trait SetStorage {
    type Main: BTreeMapTrait<Rank, FlexId>;
    type Dimension: BTreeMapTrait<Segment, RoaringTreemap>;

    fn main(&self) -> &Self::Main;
    fn main_mut(&mut self) -> &mut Self::Main;
}
