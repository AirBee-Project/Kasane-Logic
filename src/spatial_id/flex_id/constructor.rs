use crate::{Error, F_MAX, F_MIN, FlexId, MAX_ZOOM_LEVEL, SpatialIdError, TemporalId, XY_MAX};

impl FlexId {
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
            TemporalId::WHOLE,
        )
    }

    pub unsafe fn new_unchecked(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
    ) -> FlexId {
        FlexId {
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
            temporal_id: TemporalId::WHOLE,
        }
    }

    pub fn new_with_temporal(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
        temporal_id: TemporalId,
    ) -> Result<FlexId, Error> {
        if f_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z: f_zoomlevel }.into());
        }

        if x_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z: x_zoomlevel }.into());
        }

        if y_zoomlevel > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z: y_zoomlevel }.into());
        }

        if f_index < F_MIN[f_zoomlevel as usize] || f_index > F_MAX[f_zoomlevel as usize] {
            return Err(SpatialIdError::FOutOfRange {
                z: f_zoomlevel,
                f: f_index,
            }
            .into());
        }

        if x_index > XY_MAX[x_zoomlevel as usize] {
            return Err(SpatialIdError::XOutOfRange {
                z: x_zoomlevel,
                x: x_index,
            }
            .into());
        }

        if y_index > XY_MAX[y_zoomlevel as usize] {
            return Err(SpatialIdError::YOutOfRange {
                z: y_zoomlevel,
                y: y_index,
            }
            .into());
        }

        Ok(FlexId {
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,

            temporal_id,
        })
    }

    pub unsafe fn new_with_temporal_unchecked(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
        temporal_id: TemporalId,
    ) -> FlexId {
        FlexId {
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
            temporal_id,
        }
    }
}
