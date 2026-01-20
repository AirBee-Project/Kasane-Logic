use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::{
    BTreeMapTrait,
    spatial_id::{collection::Rank, flex_id::FlexId},
};

pub trait TableStorage {
    type Value: Clone + PartialEq + Ord;
    type Main: BTreeMapTrait<Rank, (FlexId, Self::Value)>;
    type Index: BTreeMapTrait<Self::Value, RoaringTreemap>;

    fn main(&self) -> &Self::Main;
    fn main_mut(&mut self) -> &mut Self::Main;

    fn index(&self) -> &Self::Index;
    fn index_mut(&mut self) -> &mut Self::Index;
}
