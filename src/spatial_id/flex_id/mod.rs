pub mod constructor;
pub mod convert;
pub mod impls;
pub mod segment;

use crate::TemporalId;

#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord)]
///拡張空間ID
pub struct FlexId {
    f_zoomlevel: u8,
    f_index: i32,
    x_zoomlevel: u8,
    x_index: u32,
    y_zoomlevel: u8,
    y_index: u32,
    temporal_id: TemporalId,
}

impl FlexId {
    pub fn f_zoomlevel(&self) -> u8 {
        self.f_zoomlevel
    }
    pub fn x_zoomlevel(&self) -> u8 {
        self.x_zoomlevel
    }
    pub fn y_zoomlevel(&self) -> u8 {
        self.f_zoomlevel
    }
    pub fn f_index(&self) -> i32 {
        self.f_index
    }
    pub fn x_index(&self) -> u32 {
        self.x_index
    }
    pub fn y_index(&self) -> u32 {
        self.y_index
    }
}
