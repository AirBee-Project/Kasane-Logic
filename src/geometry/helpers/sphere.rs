use crate::{
    Coordinate, Ecef, SingleId,
    geometry::constants::WGS84_A,
    spatial_id::{SpatialId, helpers::Dimension},
};

pub fn voxel_length(z: u8, axis: Dimension<()>) -> f64 {
    let n = 2f64.powi(z as i32);

    match axis {
        // 赤道周長 = 2πa
        Dimension::X(_) | Dimension::Y(_) => 2.0 * std::f64::consts::PI * WGS84_A / n,
        // F方向（高度）
        Dimension::F(_) => 2f64.powi(25 - z as i32),
    }
}

/// 指定された中心点と半径で定義される球状領域を覆う空間 ID を列挙する。
pub fn sphere(z: u8, center: &Coordinate, radius: f64) -> impl Iterator<Item = SingleId> {
    let voxel_diag_half = voxel_length(z, Dimension::X(())) * 3.0_f64.sqrt() / 2.0;
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
                if let Ok(id) = e.to_single_id(z) {
                    corners.push(id);
                }
            }
        }
    }

    let x_min = corners.iter().map(|v| v.as_x()).min().unwrap();
    let x_max = corners.iter().map(|v| v.as_x()).max().unwrap();
    let y_min = corners.iter().map(|v| v.as_y()).min().unwrap();
    let y_max = corners.iter().map(|v| v.as_y()).max().unwrap();
    let f_min = corners.iter().map(|v| v.as_f()).min().unwrap();
    let f_max = corners.iter().map(|v| v.as_f()).max().unwrap();

    (x_min..=x_max)
        .flat_map(move |x| {
            (y_min..=y_max).flat_map(move |y| {
                (f_min..=f_max).map(move |f| unsafe { SingleId::new_unchecked(z, f, x, y) })
            })
        })
        .filter(move |id| {
            let p: Coordinate = id.center();
            center.distance(&p) <= radius + voxel_diag_half
        })
}
