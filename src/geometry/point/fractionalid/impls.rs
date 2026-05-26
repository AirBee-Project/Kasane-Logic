use std::fmt;

use crate::{
    Coordinate, Ecef, FractionalId, Point, SpatialIdError,
    geometry::traits::CoverSingleIds,
    spatial_id::{constants::MAX_ZOOM_LEVEL, helpers},
};

impl fmt::Debug for FractionalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FractionalId")
            .field("z", &self.z)
            .field("f", &self.f)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl From<FractionalId> for Coordinate {
    /// `FractionalId` から地理座標（`Coordinate`）への変換。
    fn from(value: FractionalId) -> Self {
        let alt = helpers::altitude(value.f, value.z);
        let lat = helpers::latitude(value.y, value.z);
        let lon = helpers::longitude(value.x, value.z);
        unsafe { Coordinate::new_unchecked(lat, lon, alt) }
    }
}

impl From<FractionalId> for Ecef {
    /// `FractionalId` から地心直交座標系（`Ecef`）への変換。
    fn from(value: FractionalId) -> Self {
        let coord: Coordinate = value.into();
        coord.into()
    }
}

impl Point for FractionalId {}

impl CoverSingleIds for FractionalId {
    fn cover_single_ids(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = crate::SingleId>, crate::Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }
        let coord: Coordinate = (*self).into();
        Ok(std::iter::once(coord.single_id(z)?))
    }
}
