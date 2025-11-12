use crate::{
    point::{ECEF, Point},
    space_time_id::SpaceTimeId,
};
use std::{collections::HashSet, f64::consts::PI};

pub fn triangle(z: u8, a: Point, b: Point, c: Point) -> HashSet<SpaceTimeId> {
    let ecef_a = a.to_ecef();
    let ecef_b = b.to_ecef();
    let ecef_c = c.to_ecef();

    let coord_a = a.to_coordinate();
    let coord_b = b.to_coordinate();
    let coord_c = c.to_coordinate();

    // --- 最小緯度のラジアン値 ---
    let min_lat_rad = coord_a
        .latitude
        .abs()
        .min(coord_b.latitude.abs())
        .min(coord_c.latitude.abs())
        .to_radians();

    let r = 6_378_137.0_f64; // 地球半径
    let d = PI * r * min_lat_rad.cos() * 2f64.powi(-2 - z as i32);

    // --- 各辺の長さ ---
    let l1 = ((ecef_c.x - ecef_b.x).powi(2)
        + (ecef_c.y - ecef_b.y).powi(2)
        + (ecef_c.z - ecef_b.z).powi(2))
    .sqrt();
    let l2 = ((ecef_a.x - ecef_c.x).powi(2)
        + (ecef_a.y - ecef_c.y).powi(2)
        + (ecef_a.z - ecef_c.z).powi(2))
    .sqrt();
    let l3 = ((ecef_a.x - ecef_b.x).powi(2)
        + (ecef_a.y - ecef_b.y).powi(2)
        + (ecef_a.z - ecef_b.z).powi(2))
    .sqrt();

    let steps = (l1.max(l2).max(l3) / d).ceil() as usize;

    let mut voxels = HashSet::new();

    // --- 三角形内を走査 ---
    for i in 0..=steps {
        let t = i as f64 / steps as f64;

        // A→B 線
        let line1_x = ecef_a.x * (1.0 - t) + ecef_b.x * t;
        let line1_y = ecef_a.y * (1.0 - t) + ecef_b.y * t;
        let line1_z = ecef_a.z * (1.0 - t) + ecef_b.z * t;

        // A→C 線
        let line2_x = ecef_a.x * (1.0 - t) + ecef_c.x * t;
        let line2_y = ecef_a.y * (1.0 - t) + ecef_c.y * t;
        let line2_z = ecef_a.z * (1.0 - t) + ecef_c.z * t;

        for j in 0..=i {
            if i == 0 {
                let voxel_id = ecef_a.to_id(z);
                voxels.insert(voxel_id);
            } else {
                let s = j as f64 / i as f64;
                let x = line1_x * (1.0 - s) + line2_x * s;
                let y = line1_y * (1.0 - s) + line2_y * s;
                let z_pos = line1_z * (1.0 - s) + line2_z * s;

                let voxel_id = ECEF::new(x, y, z_pos).to_id(z);
                voxels.insert(voxel_id);
            }
        }
    }

    voxels
}
