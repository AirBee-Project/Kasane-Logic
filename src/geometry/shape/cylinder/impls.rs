use alloc::vec::Vec;

use crate::{Coordinate, CoverRangeIds, CoverSingleIds, Cylinder, Error, RangeId, Shape, SingleId};

impl Shape for Cylinder {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity([self.start, self.end])
    }
}

impl CoverSingleIds for Cylinder {
    fn cover_single_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (SingleId, V)>, Error>
    where
        V: Clone + 'static,
    {
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_single_ids_with(z, value)?.collect();
        Ok(ids.into_iter())
    }
}

impl CoverRangeIds for Cylinder {
    fn cover_range_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (RangeId, V)>, Error>
    where
        V: Clone + 'static,
    {
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_range_ids_with(z, value)?.collect();
        Ok(ids.into_iter())
    }
}
