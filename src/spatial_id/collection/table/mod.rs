use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::BTreeMapTrait;

pub trait TableStorage {
    type Value: Clone + PartialEq + Ord;
    type Index: BTreeMapTrait<Self::Value, RoaringTreemap>;

    fn index(&self) -> &Self::Index;
    fn index_mut(&mut self) -> &mut Self::Index;
}
