use crate::{Coordinate, Cylinder, Error, Geometry, IntoSolids, RangeId, Shape, SingleId};

impl Shape for Cylinder {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(&[self.start, self.end])
    }
}

impl Geometry for Cylinder {
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let solid = self.iter_solids().next().unwrap();
        let ids: Vec<_> = solid.single_ids(z)?.collect();
        Ok(ids.into_iter())
    }

    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        let solid = self.iter_solids().next().unwrap();
        let ids: Vec<_> = solid.range_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}
