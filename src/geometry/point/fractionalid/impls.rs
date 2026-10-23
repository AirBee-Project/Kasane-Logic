use core::fmt;

use crate::{
    Coordinate, Ecef, Error, FractionalId, Point, geometry::traits::CoverSingleIds, id::helpers,
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
        Coordinate::new(lat, lon, alt).unwrap()
    }
}

impl From<FractionalId> for Ecef {
    /// `FractionalId` から地心直交座標系（`Ecef`）への変換。
    fn from(value: FractionalId) -> Self {
        let coord: Coordinate = value.into();
        coord.into()
    }
}

impl Point for FractionalId {
    fn fractional_id(&self, z: impl Into<u8>) -> Result<FractionalId, Error> {
        let new = *self;
        new.into_another_zoomlevel(z)
    }
}

impl CoverSingleIds for FractionalId {
    type Value<'a> = ();

    fn cover_single_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        mut f: F,
    ) -> Result<impl Iterator<Item = (crate::SingleId, V)> + 'a, crate::Error>
    where
        V: Clone + 'a,
        F: FnMut(Self::Value<'a>) -> V + 'a,
    {
        let value = f(());
        Ok(core::iter::once((self.single_id(z)?, value)))
    }
}
