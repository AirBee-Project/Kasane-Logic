use alloc::vec::Vec;

use crate::{Coordinate, CoverRangeIds, CoverSingleIds, Cylinder, Error, RangeId, Shape, SingleId};

impl Shape for Cylinder {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity([self.start, self.end])
    }
}

impl CoverSingleIds for Cylinder {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_single_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}

impl CoverRangeIds for Cylinder {
    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_range_ids(z)?.collect();
        Ok(ids.into_iter())
    }
}
