use roaring::RoaringTreemap;

use crate::{
    BTreeMapTrait,
    spatial_id::{flex_id::FlexId, segment::Segment},
};

pub mod helpers;
pub mod map;
pub mod set;
pub mod table;

pub type Rank = u64;
const MAX_RECYCLE_CAPACITY: usize = 1024;

pub trait Collection {
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

    fn fetch_rank(&mut self) -> u64;
    fn return_rank(&mut self, rank: u64);
}
