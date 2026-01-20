use roaring::RoaringTreemap;
pub mod logic;
pub mod memory;

use crate::{
    BTreeMapTrait,
    spatial_id::{collection::Rank, flex_id::FlexId, segment::Segment},
};

pub trait MapStorage {}
