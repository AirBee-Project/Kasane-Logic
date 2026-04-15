use std::collections::HashSet;

use crate::{
    Coordinate, IntoTriangles, Polygon, RangeId, Shape, SingleId, geometry::traits::CoverSingleIds,
};

impl Shape for Polygon {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(self.vertices.clone())
    }
}

impl CoverSingleIds for Polygon {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, crate::Error> {
        let mut unique_ids = HashSet::new();

        for triangle in self.iter_triangles() {
            let ids_iter = triangle.cover_single_ids(z)?;
            for id in ids_iter {
                unique_ids.insert(id);
            }
        }

        Ok(unique_ids.into_iter())
    }
}
