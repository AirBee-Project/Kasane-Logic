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
