use core::fmt;

use crate::{
    Coordinate, Ecef, FractionalId, Point, geometry::traits::CoverSingleIds, spatial_id::helpers,
};

impl fmt::Debug for FractionalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FractionalId")
            .field("z", &self.z.get())
            .field("f", &self.f)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl From<FractionalId> for Coordinate {
    /// `FractionalId` から地理座標（`Coordinate`）への変換。
    fn from(value: FractionalId) -> Self {
        let z = value.z.get();
        let alt = helpers::altitude(value.f, z);
        let lat = helpers::latitude(value.y, z);
        let lon = helpers::longitude(value.x, z);
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
        let zoom = crate::spatial_id::zoom_level::ZoomLevel::new(z)?;
        let coord: Coordinate = (*self).into();
        Ok(core::iter::once(coord.single_id(zoom.get())?))
    }
}
