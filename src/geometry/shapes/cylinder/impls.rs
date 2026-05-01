use crate::{
    Coordinate, CoverRangeIds, CoverSingleIds, Cylinder, Error, IntoSolids, RangeId, Shape,
    SingleId,
};

impl Shape for Cylinder {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity([self.start, self.end])
    }
}

impl CoverSingleIds for Cylinder {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let solid = self.iter_solids().next().unwrap();
        let ids: Vec<_> = solid.cover_single_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}

impl CoverRangeIds for Cylinder {
    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        let solid = self.iter_solids().next().unwrap();
        let ids: Vec<_> = solid.cover_range_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}
