use std::{cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{
    Coordinate, Ecef, Error, MAX_ZOOM_LEVEL, SingleId,
    geometry::{
        constants::WGS84_A,
        point::{coordinate, ecef},
    },
};

///三角形を表す型
pub struct Triangle {
    points: [Coordinate; 3],
}

impl Triangle {
    ///[Triangle]を作成する。
    ///
    /// 3つの点が一直線上にある場合や同一の座標の場合も問題なく作成される
    pub fn new(points: [Coordinate; 3]) -> Self {
        Self { points }
    }

    ///[SingleId]の集合へ変換を行います。
    pub fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(Error::ZOutOfRange { z });
        }

        let ecef_a: Ecef = self.points[0].into();
        let ecef_b: Ecef = self.points[1].into();
        let ecef_c: Ecef = self.points[2].into();

        let min_lat_rad = self.points[0]
            .as_latitude()
            .abs()
            .min(self.points[1].as_latitude().abs())
            .min(self.points[2].as_latitude().abs())
            .to_radians();

        let d = PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-2 - z as i32);

        let l1 = ((ecef_c.as_x() - ecef_b.as_x()).powi(2)
            + (ecef_c.as_y() - ecef_b.as_y()).powi(2)
            + (ecef_c.as_z() - ecef_b.as_z()).powi(2))
        .sqrt();
        let l2 = ((ecef_a.as_x() - ecef_c.as_x()).powi(2)
            + (ecef_a.as_y() - ecef_c.as_y()).powi(2)
            + (ecef_a.as_z() - ecef_c.as_z()).powi(2))
        .sqrt();
        let l3 = ((ecef_a.as_x() - ecef_b.as_x()).powi(2)
            + (ecef_a.as_y() - ecef_b.as_y()).powi(2)
            + (ecef_a.as_z() - ecef_b.as_z()).powi(2))
        .sqrt();

        let steps = (l1.max(l2).max(l3) / d).ceil() as usize;

        let seen = Rc::new(RefCell::new(HashSet::new()));

        let iter = (0..=steps).flat_map(move |i| {
            let t = i as f64 / steps as f64;

            let line1 = (
                ecef_a.as_x() * (1.0 - t) + ecef_b.as_x() * t,
                ecef_a.as_y() * (1.0 - t) + ecef_b.as_y() * t,
                ecef_a.as_z() * (1.0 - t) + ecef_b.as_z() * t,
            );
            let line2 = (
                ecef_a.as_x() * (1.0 - t) + ecef_c.as_x() * t,
                ecef_a.as_y() * (1.0 - t) + ecef_c.as_y() * t,
                ecef_a.as_z() * (1.0 - t) + ecef_c.as_z() * t,
            );

            let seen = seen.clone();

            (0..=i).filter_map(move |j| {
                let (x, y, z_pos) = if i == 0 {
                    (ecef_a.as_x(), ecef_a.as_y(), ecef_a.as_z())
                } else {
                    let s = j as f64 / i as f64;
                    (
                        line1.0 * (1.0 - s) + line2.0 * s,
                        line1.1 * (1.0 - s) + line2.1 * s,
                        line1.2 * (1.0 - s) + line2.2 * s,
                    )
                };

                if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_single_id(z) {
                    let mut borrowed = seen.borrow_mut();
                    if borrowed.insert(voxel_id.clone()) {
                        Some(voxel_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        });

        Ok(iter)
    }

    pub fn single_ids_neo(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [[f64; 3]; 3] = [
            coordinate_to_matrix(self.points[0], z),
            coordinate_to_matrix(self.points[1], z),
            coordinate_to_matrix(self.points[2], z),
        ];
        let line1 = (0..=2)
            .map(|i| (points[0][i] - points[1][i]) * (points[0][i] - points[1][i]))
            .sum::<f64>()
            .sqrt();
        let line2 = (0..=2)
            .map(|i| (points[1][i] - points[2][i]) * (points[1][i] - points[2][i]))
            .sum::<f64>()
            .sqrt();
        let line3 = (0..=2)
            .map(|i| (points[2][i] - points[0][i]) * (points[2][i] - points[0][i]))
            .sum::<f64>()
            .sqrt();
        const D: f64 = 8.0;
        let steps = (D * line1.max(line2).max(line3).ceil()) as usize;
        for i in (0..=steps) {
            let p1 = [0, 1, 2].map(|k| {
                points[0][k] * (1.0 - (i as f64 / steps as f64))
                    + points[1][k] * (i as f64) / (steps as f64)
            });
            let p2 = [0, 1, 2].map(|k| {
                points[0][k] * (1.0 - (i as f64 / steps as f64))
                    + points[2][k] * (i as f64) / (steps as f64)
            });
            for j in (0..=i) {
                let mat_p = [0, 1, 2].map(|l| {
                    (p1[l] * (1.0 - (j as f64 / i as f64)) + p2[l] * (j as f64 / i as f64)).floor()
                        as i32
                });
                let voxel = SingleId::new(z, mat_p[0], mat_p[1] as u32, mat_p[2] as u32);
            }
        }
        let voxels: Vec<SingleId> = Vec::new();
        Ok(voxels.into_iter())
    }
}

fn coordinate_to_matrix(p: Coordinate, z: u8) -> [f64; 3] {
    let lat = p.as_latitude();
    let lon = p.as_longitude();
    let alt = p.as_altitude();

    // 空間idの高さはz=25でちょうど1mになるように定義されている
    let factor = 2_f64.powi(z as i32 - 25);
    let f = factor * alt;

    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    [f, x, y]
}
