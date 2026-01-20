use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::{
    BTreeMapTrait,
    spatial_id::{collection::Rank, flex_id::FlexId, segment::Segment},
};

pub trait MapStorage {
    type Value: Clone + PartialEq;
    type Main: BTreeMapTrait<Rank, (FlexId, Self::Value)>;

    fn main(&self) -> &Self::Main;
    fn main_mut(&mut self) -> &mut Self::Main;
}
