use alloc::vec::Vec;

use crate::{
    Coordinate, CoverRangeIds, CoverSingleIds, Cylinder, Error, RangeId, Shape, SingleId,
    SpatialIdMap,
};

impl Shape for Cylinder {
    fn center(&self) -> Coordinate {
        Coordinate::centroid([self.start, self.end])
    }
}

impl CoverSingleIds for Cylinder {
    type Value = ();

    fn cover_single_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        mut f: F,
    ) -> Result<impl Iterator<Item = (SingleId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        let value = f(&());
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_single_ids(z)?.collect();
        Ok(ids.into_iter().map(move |id| (id, value.clone())))
    }

    fn cover_map<V, F>(&self, z: impl Into<u8>, f: F) -> Result<SpatialIdMap<V>, Error>
    where
        V: PartialEq + Clone + Send + Sync,
        F: FnMut(&Self::Value) -> V,
    {
        // SingleId へ展開せず RangeId のままツリーを構築して負荷を抑える。
        let mut map = SpatialIdMap::new();
        for (id, value) in self.cover_range_ids_with(z, f)? {
            map.insert(id, value);
        }
        Ok(map)
    }
}

impl CoverRangeIds for Cylinder {
    type Value = ();

    fn cover_range_ids_with<'a, V, F>(
        &'a self,
        z: impl Into<u8>,
        mut f: F,
    ) -> Result<impl Iterator<Item = (RangeId, V)> + 'a, Error>
    where
        V: Clone + 'a,
        F: FnMut(&'a Self::Value) -> V + 'a,
    {
        let value = f(&());
        let solid = self.rough_solid();
        #[allow(clippy::needless_collect)]
        let ids: Vec<_> = solid.cover_range_ids(z)?.collect();
        Ok(ids.into_iter().map(move |id| (id, value.clone())))
    }
}
