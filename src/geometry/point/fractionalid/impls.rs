use crate::{Coordinate, Ecef, FractionalId, spatial_id::helpers};

pub trait Point: Into<Ecef> {}
impl From<FractionalId> for Coordinate {
    fn from(value: FractionalId) -> Self {
        let alt = helpers::altitude(value.f, value.z);
        let lat = helpers::latitude(value.y, value.z);
        let lon = helpers::longitude(value.x, value.z);
        unsafe { Coordinate::new_unchecked(lat, lon, alt) }
    }
}
