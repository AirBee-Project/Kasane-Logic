use crate::{CoverSingleIds, Cylinder, Error, SingleId, Sphere, Tube};
use fxhash::FxBuildHasher;
use hashbrown::HashSet;

impl CoverSingleIds for Tube {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let mut ids: HashSet<SingleId, FxBuildHasher> = Sphere::new(self.points[0], self.radius_m)?
            .cover_single_ids(z)?
            .collect();
        for coos in self.points.windows(2) {
            ids.extend(Cylinder::new(coos[0], coos[1], self.radius_m)?.cover_single_ids(z)?);
            ids.extend(Sphere::new(coos[1], self.radius_m)?.cover_single_ids(z)?);
        }
        Ok(ids.into_iter())
    }
}
