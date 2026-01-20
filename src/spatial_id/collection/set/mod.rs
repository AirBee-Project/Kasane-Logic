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

    fn f(&self) -> &Self::Dimension;
    fn f_mut(&mut self) -> &mut Self::Dimension;
    fn x(&self) -> &Self::Dimension;
    fn x_mut(&mut self) -> &mut Self::Dimension;
    fn y(&self) -> &Self::Dimension;
    fn y_mut(&mut self) -> &mut Self::Dimension;
}
