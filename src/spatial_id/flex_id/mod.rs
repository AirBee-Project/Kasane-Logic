pub mod constructor;
pub mod convert;
pub mod impls;
pub mod ops;

use crate::{MAX_ZOOM_LEVEL, Side, TemporalId};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord, Hash)]
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
    pub const UPPER_MAX: FlexId = FlexId {
        f_zoomlevel: 0,
        f_index: 0,
        x_zoomlevel: 0,
        x_index: 0,
        y_zoomlevel: 0,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
    };

    pub const LOWER_MAX: FlexId = FlexId {
        f_zoomlevel: 0,
        f_index: -1,
        x_zoomlevel: 0,
        x_index: 0,
        y_zoomlevel: 0,
        y_index: 0,
        temporal_id: TemporalId::WHOLE,
    };

    pub fn f_zoomlevel(&self) -> u8 {
        self.f_zoomlevel
    }
    pub fn x_zoomlevel(&self) -> u8 {
        self.x_zoomlevel
    }
    pub fn y_zoomlevel(&self) -> u8 {
        self.y_zoomlevel
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

    ///F方向で二つに切り分ける
    pub fn f_split(&self, side: Side) -> Option<FlexId> {
        if self.f_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            return None;
        } else {
            #[cfg(feature = "temporal_id")]
            return Some(unsafe {
                FlexId::new_with_temporal_unchecked(
                    self.f_zoomlevel() + 1,
                    self.f_index() * 2 + side as i32,
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel(),
                    self.y_index(),
                    self.temporal_id.clone(),
                )
            });

            #[cfg(not(feature = "temporal_id"))]
            return Some(unsafe {
                FlexId::new_unchecked(
                    self.f_zoomlevel() + 1,
                    self.f_index() * 2 + side as i32,
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel(),
                    self.y_index(),
                )
            });
        }
    }

    ///X方向で二つに切り分ける
    pub fn x_split(&self, side: Side) -> Option<FlexId> {
        if self.x_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            return None;
        } else {
            #[cfg(feature = "temporal_id")]
            return Some(unsafe {
                FlexId::new_with_temporal_unchecked(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel() + 1,
                    self.x_index() * 2 + side as u32,
                    self.y_zoomlevel(),
                    self.y_index(),
                    self.temporal().clone(),
                )
            });

            #[cfg(not(feature = "temporal_id"))]
            return Some(unsafe {
                FlexId::new_unchecked(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel() + 1,
                    self.x_index() * 2 + side as u32,
                    self.y_zoomlevel(),
                    self.y_index(),
                )
            });
        }
    }

    ///Y方向で二つに切り分ける
    pub fn y_split(&self, side: Side) -> Option<FlexId> {
        if self.y_zoomlevel() == MAX_ZOOM_LEVEL as u8 {
            return None;
        } else {
            #[cfg(feature = "temporal_id")]
            return Some(unsafe {
                FlexId::new_with_temporal_unchecked(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel() + 1,
                    self.y_index() * 2 + side as u32,
                    self.temporal().clone(),
                )
            });

            #[cfg(not(feature = "temporal_id"))]
            return Some(unsafe {
                FlexId::new_unchecked(
                    self.f_zoomlevel(),
                    self.f_index(),
                    self.x_zoomlevel(),
                    self.x_index(),
                    self.y_zoomlevel() + 1,
                    self.y_index() * 2 + side as u32,
                )
            });
        }
    }
}
