#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::{
    Coordinate, Error, ExpandCoordinates, Shape, SingleId, Triangle, Vec3, Vec3FractionalId,
    geometry::traits::CoverSingleIds,
};

impl Shape for Triangle {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(self.expand_coordinates())
    }
}

impl CoverSingleIds for Triangle {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [Vec3FractionalId; 3] = [
            Vec3FractionalId::from(self.points[0].fractional_id(z)?),
            Vec3FractionalId::from(self.points[1].fractional_id(z)?),
            Vec3FractionalId::from(self.points[2].fractional_id(z)?),
        ];
        let diff_f = libm::floor(points[0].a().max(points[1].a()).max(points[2].a()))
            - libm::floor(points[0].a().min(points[1].a()).min(points[2].a()));
        let diff_x = libm::floor(points[0].b().max(points[1].b()).max(points[2].b()))
            - libm::floor(points[0].b().min(points[1].b()).min(points[2].b()));
        let diff_y = libm::floor(points[0].c().max(points[1].c()).max(points[2].c()))
            - libm::floor(points[0].c().min(points[1].c()).min(points[2].c()));
        let steps = libm::ceil(diff_f.max(diff_x).max(diff_y) / 8.0) as u32;
        let mut seen = HashSet::new();
        let voxels = self
            .divide(steps)?
            .flat_map(move |tri| tri.single_ids_limited(z).ok().into_iter().flatten())
            .filter(move |voxel| seen.insert(voxel.clone()));
        Ok(voxels)
    }
}
