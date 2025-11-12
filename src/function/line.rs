use std::{collections::HashSet, f64::consts::PI};

use crate::{
    point::{ECEF, Point},
    space_time_id::SpaceTimeId,
};

pub fn line(z: u8, a: Point, b: Point) -> HashSet<SpaceTimeId> {
    let ecef_a = a.to_ecef();
    let ecef_b = b.to_ecef();

    let coordinate_a = a.to_coordinate();
    let coordinate_b = b.to_coordinate();

    // --- ステップ計算 ---
    let dx = ecef_a.x - ecef_b.x;
    let dy = ecef_a.y - ecef_b.y;
    let dz = ecef_a.z - ecef_b.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    // 最小緯度のラジアン値
    let min_lat_rad = coordinate_a
        .latitude
        .abs()
        .min(coordinate_b.latitude.abs())
        .to_radians();
    let r = 6_378_137.0_f64; // 地球半径（WGS84）
    let d = PI * r * min_lat_rad.cos() * 2f64.powi(-3 - z as i32);

    let steps = (distance / d).ceil() as usize;
    let mut voxels = HashSet::new();

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = ecef_a.x * (1.0 - t) + ecef_b.x * t;
        let y = ecef_a.y * (1.0 - t) + ecef_b.y * t;
        let z_pos = ecef_a.z * (1.0 - t) + ecef_b.z * t;

        let voxel_id = ECEF::new(x, y, z_pos).to_id(z);
        voxels.insert(voxel_id);
    }

    voxels
}
