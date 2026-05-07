use std::collections::HashSet;

use crate::{
    Error, Shape, SingleId, Triangle,
    geometry::{shape::triangle::coordinate_to_matrix, traits::CoverSingleIds},
};

impl Shape for Triangle {}

impl CoverSingleIds for Triangle {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [[f64; 3]; 3] = [
            coordinate_to_matrix(self.points[0], z),
            coordinate_to_matrix(self.points[1], z),
            coordinate_to_matrix(self.points[2], z),
        ];
        let diff_f = points[0][0].max(points[1][0]).max(points[2][0]).floor()
            - points[0][0].min(points[1][0]).min(points[2][0]).floor();
        let diff_x = points[0][1].max(points[1][1]).max(points[2][1]).floor()
            - points[0][1].min(points[1][1]).min(points[2][1]).floor();
        let diff_y = points[0][2].max(points[1][2]).max(points[2][2]).floor()
            - points[0][2].min(points[1][2]).min(points[2][2]).floor();
        let steps = (diff_f.max(diff_x).max(diff_y) / 8.0).ceil() as u32;
        let mut seen = HashSet::new();
        let voxels = self
            .divide(steps)?
            .flat_map(move |tri| tri.single_ids_limited(z).ok().into_iter().flatten())
            .filter(move |voxel| seen.insert(voxel.clone()));
        Ok(voxels)
    }
}
