use crate::{CoverRangeIds, CoverSingleIds, Cylinder, Error, RangeId, Shape, SingleId};

impl Shape for Cylinder {}

impl CoverSingleIds for Cylinder {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let solid = self.rough_solid();
        let ids: Vec<_> = solid.cover_single_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}

impl CoverRangeIds for Cylinder {
    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        let solid = self.rough_solid();
        let ids: Vec<_> = solid.cover_range_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}
