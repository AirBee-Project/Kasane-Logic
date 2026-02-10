use std::{cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{Coordinate, Ecef, Error, MAX_ZOOM_LEVEL, SingleId, geometry::constants::WGS84_A};

///三角形を表す型
pub struct Triangle {
    points: [Coordinate; 3],
}

impl Triangle {
    ///[Triangle]を作成する。
    ///
    ///3点が同一の直線に存在する場合はエラーとなる
    pub fn new(points: [Coordinate; 3]) -> Result<Self, Error> {
        if Self::is_collinear(&points[0], &points[1], &points[2]) {
            return Err(Error::CollinearPoints);
        }
        Ok(Self { points })
    }

    ///チェックすることなく、[Triangle]を作成する。
    pub unsafe fn new_unchecked(points: [Coordinate; 3]) -> Self {
        Self { points }
    }

    pub fn points(&self) -> &[Coordinate; 3] {
        &self.points
    }

    ///同一平面上にあるかを判定する
    fn is_collinear(p0: &Coordinate, p1: &Coordinate, p2: &Coordinate) -> bool {
        let e0: Ecef = (*p0).into();
        let e1: Ecef = (*p1).into();
        let e2: Ecef = (*p2).into();

        let v1 = (
            e1.as_x() - e0.as_x(),
            e1.as_y() - e0.as_y(),
            e1.as_z() - e0.as_z(),
        );
        let v2 = (
            e2.as_x() - e0.as_x(),
            e2.as_y() - e0.as_y(),
            e2.as_z() - e0.as_z(),
        );

        // 外積 (Cross Product) を計算
        let cx = v1.1 * v2.2 - v1.2 * v2.1;
        let cy = v1.2 * v2.0 - v1.0 * v2.2;
        let cz = v1.0 * v2.1 - v1.1 * v2.0;

        // 外積の大きさの2乗
        let cross_product_sq = cx * cx + cy * cy + cz * cz;

        //浮動小数点の誤差
        cross_product_sq < f64::EPSILON
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
}
