use std::collections::HashSet;

use crate::{Coordinate, Geometry, IntoTriangles, Polygon, RangeId, Shape, SingleId};

impl Shape for Polygon {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(self.vertices.clone())
    }
}

impl Geometry for Polygon {
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, crate::Error> {
        let mut unique_ids = HashSet::new();

        for triangle in self.iter_triangles() {
            let ids_iter = triangle.single_ids(z)?;
            for id in ids_iter {
                unique_ids.insert(id);
            }
        }

        Ok(unique_ids.into_iter())
    }

    ///[SingleId]を変換しているだけなので、型の問題がなければ`fn single_ids`を使ったほうが良い
    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, crate::Error> {
        Ok(self.single_ids(z)?.map(RangeId::from))
    }
}
