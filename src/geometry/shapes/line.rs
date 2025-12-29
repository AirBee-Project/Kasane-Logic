use std::collections::HashSet;

use crate::{
    error::Error,
    geometry::{constants::WGS84_A, coordinate::Coordinate, ecef::Ecef},
    id::space_id::{constants::MAX_ZOOM_LEVEL, single::SingleID},
};
pub fn line(z: u8, a: Coordinate, b: Coordinate) -> Result<impl Iterator<Item = SingleID>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }

    // ECEF 座標に変換
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();

    // ステップ数計算
    let dx = ecef_b.as_x() - ecef_a.as_x();
    let dy = ecef_b.as_y() - ecef_a.as_y();
    let dz = ecef_b.as_z() - ecef_a.as_z();
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    let min_lat_rad = a
        .as_latitude()
        .abs()
        .min(b.as_latitude().abs())
        .to_radians();
    let d = std::f64::consts::PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-3 - z as i32);
    let steps = (distance / d).ceil() as usize;

    let t_iter = (0..=steps).map(move |i| i as f64 / steps as f64);

    let mut seen = HashSet::new();

    let iter = t_iter.filter_map(move |t| {
        let x = ecef_a.as_x() * (1.0 - t) + ecef_b.as_x() * t;
        let y = ecef_a.as_y() * (1.0 - t) + ecef_b.as_y() * t;
        let z_pos = ecef_a.as_z() * (1.0 - t) + ecef_b.as_z() * t;

        if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_id(z) {
            if seen.insert(voxel_id.clone()) {
                Some(voxel_id)
            } else {
                None
            }
        } else {
            None
        }
    });

    Ok(iter)
}

fn coordinate_to_matrix(p: Coordinate, z: u8) -> [f64; 3] {
    let lat = p.as_latitude();
    let lon = p.as_longitude();
    let alt = p.as_altitude();

    // ---- 高度 h -> f (Python の h_to_f を Rust に移植) ----
    let factor = 2_f64.powi(z as i32 - 25); // 2^(z-25)
    let f = factor * alt;

    // ---- 経度 lon -> x ----
    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    // ---- 緯度 lat -> y (Web Mercator) ----
    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    [f, x, y]
}

fn line_DDA(z: u8, a: Coordinate, b: Coordinate) -> Result<impl Iterator<Item = SingleID>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }
    let vp1 = coordinate_to_matrix(a, z);
    let vp2 = coordinate_to_matrix(b, z);
    let df_total = vp2[0] - vp1[0];
    let dx_total = vp2[1] - vp1[1];
    let dy_total = vp2[2] - vp1[2];
    let id1 = SingleID::new(
        z,
        vp1[0].floor() as i64,
        vp1[1].floor() as u64,
        vp1[2].floor() as u64,
    );
    let id2 = SingleID::new(
        z,
        vp2[0].floor() as i64,
        vp2[1].floor() as u64,
        vp2[2].floor() as u64,
    );
    let length: f64 = (df_total * df_total + dx_total * dx_total + dy_total * dy_total).sqrt();
    let df = length / df_total;
    let dx = length / dx_total;
    let dy = length / dy_total;
    let mut voxels: Vec<SingleID> = Vec::new();
    let iter = voxels.into_iter();
    Ok(iter)
}
