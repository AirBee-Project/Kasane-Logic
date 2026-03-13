use std::collections::HashSet;

use crate::{Geometry, Polygon, RangeId};

impl Geometry for Polygon {
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::SingleId>, crate::Error> {
        let mut unique_ids = HashSet::new();

        let triangles = self.triangles();
        for triangle in triangles {
            let ids_iter = triangle.single_ids(z)?;
            for id in ids_iter {
                unique_ids.insert(id);
            }
        }

        Ok(unique_ids.into_iter())
    }

    ///[SingleId]を変換しているだけなので、型の問題がなければ`fn single_ids`を使ったほうが良い
    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::RangeId>, crate::Error> {
        Ok(self.single_ids(z)?.map(|id| RangeId::from(id)))
    }
}
