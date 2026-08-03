use alloc::vec::Vec;

use crate::{
    Coordinate, Ecef, Shape, SingleId, SpatialId, Sphere, WGS84_A, geometry::traits::CoverSingleIds,
};

impl Shape for Sphere {
    fn center(&self) -> Coordinate {
        self.center
    }
}

impl CoverSingleIds for Sphere {
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, crate::Error> {
        let zoom = crate::spatial_id::zoom_level::ZoomLevel::new(z)?;
        let z = zoom.get();

        let center = self.center;
        let radius = self.radius_m;

        let voxel_diag_half = voxel_length_xy(z) * libm::sqrt(3.0) / 2.0;
        let center_ecef: Ecef = (center).into();

        // 球の8頂点 -> 探索範囲推定
        let mut corners = Vec::with_capacity(8);
        for &sx in &[1.0, -1.0] {
            for &sy in &[1.0, -1.0] {
                for &sz in &[1.0, -1.0] {
                    let e = Ecef::new(
                        center_ecef.x() + radius * sx,
                        center_ecef.y() + radius * sy,
                        center_ecef.z() + radius * sz,
                    );
                    if let Ok(id) = e.single_id(z) {
                        corners.push(id);
                    }
                }
            }
        }

        let x_min = corners.iter().map(|v| v.x()).min().unwrap();
        let x_max = corners.iter().map(|v| v.x()).max().unwrap();
        let y_min = corners.iter().map(|v| v.y()).min().unwrap();
        let y_max = corners.iter().map(|v| v.y()).max().unwrap();
        let f_min = corners.iter().map(|v| v.f()).min().unwrap();
        let f_max = corners.iter().map(|v| v.f()).max().unwrap();

        Ok((x_min..=x_max)
            .flat_map(move |x| {
                (y_min..=y_max).flat_map(move |y| {
                    (f_min..=f_max).map(move |f| SingleId::new(z, f, x, y).unwrap())
                })
            })
            .filter(move |id| {
                let p: Coordinate = id.spatial_center();
                center.distance(&p) <= radius + voxel_diag_half
            }))
    }
}

/// ズームレベル `z` における、水平方向（X/Y、値は同じ）のボクSegment1辺の長さ`[m]`。
///
/// 赤道周長 `2πa` をズームレベルで等分した値。
pub fn voxel_length_xy(z: u8) -> f64 {
    let n = libm::pow(2_f64, (z as i32) as f64);
    2.0 * core::f64::consts::PI * WGS84_A / n
}

/// ズームレベル `z` における、F方向（高度）のボクSegment1辺の長さ`[m]`。
pub fn voxel_length_f(z: u8) -> f64 {
    libm::pow(2_f64, (25 - z as i32) as f64)
}
