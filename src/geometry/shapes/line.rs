use std::{collections::HashSet, f64};

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
    let factor = 2_f64.powi(z as i32 - 25); // 空間idの高さはz=25でちょうど1mになるように定義されている
    let f = factor * alt;

    // ---- 経度 lon -> x ----
    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    // ---- 緯度 lat -> y (Web Mercator) ----
    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    [f, x, y]
}

pub fn line_dda(
    z: u8,
    a: Coordinate,
    b: Coordinate,
) -> Result<impl Iterator<Item = SingleID>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }
    let vp1 = coordinate_to_matrix(a, z);
    let vp2 = coordinate_to_matrix(b, z);
    let df_total = (vp2[0] - vp1[0]).abs();
    let dx_total = (vp2[1] - vp1[1]).abs();
    let dy_total = (vp2[2] - vp1[2]).abs();
    let i1 = vp1[0].floor() as i64;
    let j1 = vp1[1].floor() as i64;
    let k1 = vp1[2].floor() as i64;
    let i2 = vp2[0].floor() as i64;
    let j2 = vp2[1].floor() as i64;
    let k2 = vp2[2].floor() as i64;
    let length: f64 = (df_total * df_total + dx_total * dx_total + dy_total * dy_total).sqrt();
    let df = if vp2[0] != vp1[0] {
        length / df_total
    } else {
        f64::INFINITY
    };
    let dx = if vp2[1] != vp1[1] {
        length / dx_total
    } else {
        f64::INFINITY
    };
    let dy = if vp2[2] != vp1[2] {
        length / dy_total
    } else {
        f64::INFINITY
    };
    let mut tf = if i2 > i1 {
        (1.0 - vp1[0] + vp1[0].floor()) * df
    } else if i2 == i1 {
        f64::INFINITY
    } else {
        (vp1[0] - vp1[0].floor()) * df
    };
    let mut tx = if j2 > j1 {
        (1.0 - vp1[1] + vp1[1].floor()) * dx
    } else if j2 == j1 {
        f64::INFINITY
    } else {
        (vp1[1] - vp1[1].floor()) * dx
    };
    let mut ty = if k2 > k1 {
        (1.0 - vp1[2] + vp1[2].floor()) * dy
    } else if k2 == k1 {
        f64::INFINITY
    } else {
        (vp1[2] - vp1[2].floor()) * dy
    };
    let mut voxels: Vec<SingleID> = Vec::new();
    voxels.push(SingleID::new(z, i1, j1 as u64, k1 as u64)?);
    let mut current_f = i1;
    let mut current_x = j1;
    let mut current_y = k1;
    let sign_i = (vp2[0] - vp1[0]).signum() as i64;
    let sign_j = (vp2[1] - vp1[1]).signum() as i64;
    let sign_k = (vp2[2] - vp1[2]).signum() as i64;
    while current_f != i2 || current_x != j2 || current_y != k2 {
        if tf > tx {
            if ty > tx {
                tx += dx;
                current_x += sign_j;
            } else {
                ty += dy;
                current_y += sign_k;
            }
        } else {
            if tf > ty {
                ty += dy;
                current_y += sign_k;
            } else {
                tf += df;
                current_f += sign_i;
            }
        }
        voxels.push(SingleID::new(
            z,
            current_f,
            current_x as u64,
            current_y as u64,
        )?);
    }
    let iter = voxels.into_iter();
    Ok(iter)
}
