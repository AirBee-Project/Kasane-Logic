pub mod impls;
pub mod segment;

#[cfg(feature = "temporal")]
use create::TemporalId;

use crate::{Error, F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};

#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord)]
///拡張空間ID
pub struct FlexId {
    f_zoomlevel: u8,
    f_index: i32,
    x_zoomlevel: u8,
    x_index: u32,
    y_zoomlevel: u8,
    y_index: u32,
    #[cfg(feature = "temporal")]
    temporal_id: TemporalId,
}

impl FlexId {
    /// 新しく[FlexId]を作成する。
    pub fn new(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
    ) -> Result<FlexId, Error> {
        Self::new_with_temporal(
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
            #[cfg(feature = "temporal")]
            TemporalId::whole(),
        )
    }

    /// 新しく[FlexId]を作成する。
    pub fn new_with_temporal(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
        #[cfg(feature = "temporal")] temporal_id: TemporalId,
    ) -> Result<FlexId, Error> {
        if f_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::ZOutOfRange { z: f_zoomlevel });
        }

        if x_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::ZOutOfRange { z: x_zoomlevel });
        }

        if y_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::ZOutOfRange { z: y_zoomlevel });
        }

        if f_index < F_MIN[f_zoomlevel as usize] || f_index > F_MAX[f_zoomlevel as usize] {
            return Err(Error::FOutOfRange {
                z: f_zoomlevel,
                f: f_index,
            });
        }

        if x_index > XY_MAX[x_zoomlevel as usize] {
            return Err(Error::XOutOfRange {
                z: x_zoomlevel,
                x: x_index,
            });
        }

        if y_index > XY_MAX[y_zoomlevel as usize] {
            return Err(Error::YOutOfRange {
                z: y_zoomlevel,
                y: y_index,
            });
        }

        Ok(FlexId {
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
            #[cfg(feature = "temporal")]
            temporal_id,
        })
    }

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

    #[inline]
    pub fn get_f_bit(&self, depth: u8) -> u8 {
        debug_assert!(depth < self.f_zoomlevel);
        let shift = (self.f_zoomlevel - 1) - depth;
        ((self.f_index >> shift) & 1) as u8
    }

    #[inline]
    pub fn get_x_bit(&self, depth: u8) -> u8 {
        debug_assert!(depth < self.x_zoomlevel);
        let shift = (self.x_zoomlevel - 1) - depth;
        ((self.x_index >> shift) & 1) as u8
    }

    #[inline]
    pub fn get_y_bit(&self, depth: u8) -> u8 {
        debug_assert!(depth < self.y_zoomlevel);
        let shift = (self.y_zoomlevel - 1) - depth;
        ((self.y_index >> shift) & 1) as u8
    }
}
