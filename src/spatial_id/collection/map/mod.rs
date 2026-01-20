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
    type Dimension: BTreeMapTrait<Segment, RoaringTreemap>;

    fn main(&self) -> &Self::Main;
    fn main_mut(&mut self) -> &mut Self::Main;

    fn f(&self) -> &Self::Dimension;
    fn f_mut(&mut self) -> &mut Self::Dimension;
    fn x(&self) -> &Self::Dimension;
    fn x_mut(&mut self) -> &mut Self::Dimension;
    fn y(&self) -> &Self::Dimension;
    fn y_mut(&mut self) -> &mut Self::Dimension;
}
