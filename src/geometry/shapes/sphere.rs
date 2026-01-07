use crate::{
    geometry::{constants::WGS84_A, coordinate::Coordinate, ecef::Ecef},
    spatial_id::{SpatialId, single::SingleId},
};

#[derive(Debug, Clone, Copy)]
pub enum VoxelAxis {
    X,
    Y,
    F,
}

impl Coordinate {
    pub fn distance(&self, other: &Coordinate) -> f64 {
        let e1: Ecef = (*self).into();
        let e2: Ecef = (*other).into();
        ((e1.as_x() - e2.as_x()).powi(2)
            + (e1.as_y() - e2.as_y()).powi(2)
            + (e1.as_z() - e2.as_z()).powi(2))
        .sqrt()
    }
}

/// ===============================
/// voxel 1辺の長さ（m）
/// ===============================
pub fn voxel_length(z: u8, axis: VoxelAxis) -> f64 {
    let n = 2f64.powi(z as i32);

    match axis {
        // 赤道周長 = 2πa
        VoxelAxis::X | VoxelAxis::Y => 2.0 * std::f64::consts::PI * WGS84_A / n,
        // F方向（高度）
        VoxelAxis::F => 2f64.powi(25 - z as i32),
    }
}

pub fn sphere<'a>(
    z: u8,
    center: &'a Coordinate,
    radius: f64,
) -> impl Iterator<Item = SingleId> + 'a {
    let voxel_diag_half = voxel_length(z, VoxelAxis::X) * 3.0_f64.sqrt() / 2.0;
    let center_ecef: Ecef = (*center).into();

    // 球の8頂点 → 探索範囲推定
    let mut corners = Vec::with_capacity(8);
    for &sx in &[1.0, -1.0] {
        for &sy in &[1.0, -1.0] {
            for &sz in &[1.0, -1.0] {
                let e = Ecef::new(
                    center_ecef.as_x() + radius * sx,
                    center_ecef.as_y() + radius * sy,
                    center_ecef.as_z() + radius * sz,
                );
                if let Ok(id) = e.to_id(z) {
                    corners.push(id);
                }
            }
        }
    }

    let x_min = corners.iter().map(|v| v.x).min().unwrap();
    let x_max = corners.iter().map(|v| v.x).max().unwrap();
    let y_min = corners.iter().map(|v| v.y).min().unwrap();
    let y_max = corners.iter().map(|v| v.y).max().unwrap();
    let f_min = corners.iter().map(|v| v.f).min().unwrap();
    let f_max = corners.iter().map(|v| v.f).max().unwrap();

    (x_min..=x_max)
        .flat_map(move |x| {
            (y_min..=y_max).flat_map(move |y| {
                (f_min..=f_max).map(move |f| unsafe { SingleId::uncheck_new(z, f, x, y) })
            })
        })
        .filter(move |id| {
            let p: Coordinate = id.center();
            center.distance(&p) <= radius + voxel_diag_half
        })
}
