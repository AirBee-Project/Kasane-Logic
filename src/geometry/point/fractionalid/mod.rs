use crate::{
    Ecef, SingleId,
    error::{Error, GeometryError, SpatialIdError},
    spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
};
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FractionalId {
    z: u8,
    f: f64,
    x: f64,
    y: f64,
}
impl FractionalId {
    pub fn new(z: u8, f: f64, x: f64, y: f64) -> Result<Self, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize] as f64;
        let f_max = F_MAX[z as usize] as f64;
        let xy_max = XY_MAX[z as usize] as f64;

        if f < f_min || f > f_max {
            return Err(SpatialIdError::FOutOfRange {
                f: f.floor() as i32,
                z,
            }
            .into());
        }
        if x > xy_max {
            return Err(SpatialIdError::XOutOfRange {
                x: x.floor() as u32,
                z,
            }
            .into());
        }
        if y > xy_max {
            return Err(SpatialIdError::YOutOfRange {
                y: y.floor() as u32,
                z,
            }
            .into());
        }

        Ok(FractionalId { z, f, x, y })
    }
}
