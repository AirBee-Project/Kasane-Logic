use std::{collections::HashSet, f64::consts::PI, iter};

use crate::{
    geometry::{constants::WGS84_A, coordinate::Coordinate, ecef::Ecef},
    id::space_id::single::SingleID,
};

pub fn line(z: u8, a: Coordinate, b: Coordinate) -> impl Iterator<Item = SingleID> {
    // ECEF 座標に変換
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();

    // ステップ数計算
    let dx = ecef_b.x - ecef_a.x;
    let dy = ecef_b.y - ecef_a.y;
    let dz = ecef_b.z - ecef_a.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    let min_lat_rad = a.latitude.abs().min(b.latitude.abs()).to_radians();
    let d = std::f64::consts::PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-3 - z as i32);
    let steps = (distance / d).ceil() as usize;

    // ステップごとの t 値を作るイテレータ
    let t_iter = (0..=steps).map(move |i| i as f64 / steps as f64);

    // すでに生成済みの SingleID を保持する HashSet
    let mut seen = HashSet::new();

    t_iter.filter_map(move |t| {
        let x = ecef_a.x * (1.0 - t) + ecef_b.x * t;
        let y = ecef_a.y * (1.0 - t) + ecef_b.y * t;
        let z_pos = ecef_a.z * (1.0 - t) + ecef_b.z * t;

        if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_id(z) {
            if seen.insert(voxel_id.clone()) {
                Some(voxel_id)
            } else {
                None
            }
        } else {
            None
        }
    })
}
