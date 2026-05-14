use crate::geometry::shape::triangle::OldTriangle;
use std::collections::HashSet;

use crate::{
    Coordinate, Ecef, Error, ExpandCoordinates, MAX_ZOOM_LEVEL, Shape, SingleId, SpatialIdError,
    Triangle,
    geometry::{shape::triangle::coordinate_to_matrix, traits::CoverSingleIds},
};

impl Shape for Triangle {
    fn center(&self) -> Coordinate {
        Coordinate::center_gravity(self.expand_coordinates())
    }
}

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

impl CoverSingleIds for OldTriangle {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }
        let a = self.points[0];
        let b = self.points[1];
        let c = self.points[2];
        let ecef_a: Ecef = a.into();
        let ecef_b: Ecef = b.into();
        let ecef_c: Ecef = c.into();

        let minlat = a
            .latitude()
            .abs()
            .min(b.latitude().abs())
            .min(c.latitude().abs())
            .to_radians();
        let r = 6378137.0;

        // ボクセルの東西距離の1/8
        let d = std::f64::consts::PI * r * minlat.cos() * (2.0f64).powi(-2 - (z as i32));

        // 3辺の長さを計算し、最大値でstepsを決定
        let l1 = ecef_b.distance(&ecef_c);
        let l2 = ecef_c.distance(&ecef_a);
        let l3 = ecef_a.distance(&ecef_b);
        let steps = (l1.max(l2).max(l3) / d).ceil() as usize;

        let mut voxels = std::collections::HashSet::new();

        for i in 0..=steps {
            if i == 0 {
                // 頂点Aのボクセルを追加
                let single_id = a.single_id(z)?;
                voxels.insert(single_id);
            } else {
                let t = i as f64 / steps as f64;
                // A→B 上の点
                let line1_pt: Coordinate = Ecef::new(
                    ecef_a.x() * (1.0 - t) + ecef_b.x() * t,
                    ecef_a.y() * (1.0 - t) + ecef_b.y() * t,
                    ecef_a.z() * (1.0 - t) + ecef_b.z() * t,
                )
                .try_into()?;
                // A→C 上の点
                let line2_pt: Coordinate = Ecef::new(
                    ecef_a.x() * (1.0 - t) + ecef_c.x() * t,
                    ecef_a.y() * (1.0 - t) + ecef_c.y() * t,
                    ecef_a.z() * (1.0 - t) + ecef_c.z() * t,
                )
                .try_into()?;
                // line1[i] → line2[i] をoldlineでカバー
                for single_id in oldline(line1_pt, line2_pt, z)? {
                    voxels.insert(single_id);
                }
            }
        }

        Ok(voxels.into_iter())
    }
}
fn oldline(a: Coordinate, b: Coordinate, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(SpatialIdError::ZOutOfRange { z }.into());
    }

    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();

    let a1 = ecef_a.x();
    let b1 = ecef_a.y();
    let c1 = ecef_a.z();

    let a2 = ecef_b.x();
    let b2 = ecef_b.y();
    let c2 = ecef_b.z();

    let minlat = a.latitude().abs().min(b.latitude().abs()).to_radians();
    let r = 6378137.0;

    // ボクセルの東西距離の1/8
    let d = std::f64::consts::PI * r * minlat.cos() * (2.0f64).powi(-2 - (z as i32));

    let distance = ((a1 - a2).powi(2) + (b1 - b2).powi(2) + (c1 - c2).powi(2)).sqrt();
    let steps = (distance / d).ceil().max(1.0) as usize;

    let mut voxels = std::collections::HashSet::new();

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = a1 * (1.0 - t) + a2 * t;
        let y = b1 * (1.0 - t) + b2 * t;
        let z_pos = c1 * (1.0 - t) + c2 * t;

        let point: Coordinate = Ecef::new(x, y, z_pos).try_into()?;
        let single_id = point.single_id(z)?;

        voxels.insert(single_id);
    }

    Ok(voxels.into_iter())
}
