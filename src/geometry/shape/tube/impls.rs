use crate::{CoverSingleIds, Cylinder, Error, SingleId, Sphere, Tube};
use hashbrown::HashSet;

impl CoverSingleIds for Tube {
    type Value = ();
    fn cover_single_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (SingleId, Self::Value)>, Error> {
        let mut ids: HashSet<_> = Sphere::new(self.points[0], self.radius_m)?
            .cover_single_ids(z)?
            .collect();
        for coos in self.points.windows(2) {
            ids.extend(Cylinder::new(coos[0], coos[1], self.radius_m)?.cover_single_ids(z)?);
            ids.extend(Sphere::new(coos[1], self.radius_m)?.cover_single_ids(z)?);
        }
        Ok(ids.into_iter().map(|id| (id, ())))
    }
}
