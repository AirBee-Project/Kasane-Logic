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
    let d_o1 = ecef_b.as_x() - ecef_a.as_x();
    let d_o2 = ecef_b.as_y() - ecef_a.as_y();
    let dz = ecef_b.as_z() - ecef_a.as_z();
    let distance = (d_o1 * d_o1 + d_o2 * d_o2 + dz * dz).sqrt();

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
    let d_total = [
        (vp2[0] - vp1[0]).abs(),
        (vp2[0] - vp1[0]).abs(),
        (vp2[2] - vp1[2]).abs(),
    ];
    let max_d = d_total[0].max(d_total[1]).max(d_total[2]);
    let max_flag: usize = if max_d == d_total[0] {
        0
    } else if max_d == d_total[1] {
        1
    } else {
        2
    };
    let other_flag_1 = (max_flag + 1) % 3;
    let other_flag_2 = (max_flag + 2) % 3;
    let i1 = vp1[max_flag].floor() as i64;
    let j1 = vp1[other_flag_1].floor() as i64;
    let k1 = vp1[other_flag_2].floor() as i64;
    let i2 = vp2[max_flag].floor() as i64;
    let j2 = vp2[other_flag_1].floor() as i64;
    let k2 = vp2[other_flag_2].floor() as i64;
    let d_o1 = if vp2[other_flag_1] != vp1[other_flag_1] {
        d_total[max_flag] / d_total[other_flag_1]
    } else {
        f64::INFINITY
    };
    let d_o2 = if vp2[other_flag_2] != vp1[other_flag_2] {
        d_total[max_flag] / d_total[other_flag_2]
    } else {
        f64::INFINITY
    };
    let tm = if i2 > i1 {
        1.0 - vp1[other_flag_1] + vp1[other_flag_1].floor()
    } else if i2 == i1 {
        f64::INFINITY
    } else {
        vp1[other_flag_1] - vp1[other_flag_1].floor()
    };
    let mut to1 = if j2 > j1 {
        (1.0 - vp1[other_flag_1] + vp1[other_flag_1].floor()) * d_o1 - tm
    } else if j2 == j1 {
        f64::INFINITY
    } else {
        (vp1[other_flag_1] - vp1[other_flag_1].floor()) * d_o1 - tm
    };
    let mut to2 = if k2 > k1 {
        (1.0 - vp1[2] + vp1[2].floor()) * d_o2 - tm
    } else if k2 == k1 {
        f64::INFINITY
    } else {
        (vp1[2] - vp1[2].floor()) * d_o2 - tm
    };
    let mut tm_int: u16 = 0;
    let max_steps = (i2 - i1).abs() as u16;
    let mut voxels: Vec<SingleID> = Vec::new();
    voxels.push(SingleID::new(z, i1, j1 as u64, k1 as u64)?);
    let mut current = [i1, j1, k1];
    let sign_i = (vp2[max_flag] - vp1[max_flag]).signum() as i64;
    let sign_j = (vp2[other_flag_1] - vp1[other_flag_1]).signum() as i64;
    let sign_k = (vp2[other_flag_2] - vp1[other_flag_2]).signum() as i64;
    let mut steps = 0;
    while current != [i2, j2, k2] {
        steps += 1;
        if to1 > to2 {
            if tm_int as f64 > to2 {
                to2 += d_o2;
                current[2] += sign_k;
                voxels.push(SingleID::new(
                    z,
                    current[3 - max_flag],
                    current[3 - other_flag_2] as u64,
                    current[3 - other_flag_1] as u64,
                )?);
                if (tm_int as f64) > to1 {
                    to1 += d_o2;
                    current[1] += sign_j;
                    voxels.push(SingleID::new(
                        z,
                        current[3 - max_flag],
                        current[3 - other_flag_2] as u64,
                        current[3 - other_flag_1] as u64,
                    )?);
                }
            }
        } else {
            if tm_int as f64 > to1 {
                to1 += d_o1;
                current[1] += sign_j;
                voxels.push(SingleID::new(
                    z,
                    current[3 - max_flag],
                    current[3 - other_flag_2] as u64,
                    current[3 - other_flag_1] as u64,
                )?);
                if (tm_int as f64) > to2 {
                    to2 += d_o2;
                    current[2] += sign_k;
                    voxels.push(SingleID::new(
                        z,
                        current[3 - max_flag],
                        current[3 - other_flag_2] as u64,
                        current[3 - other_flag_1] as u64,
                    )?);
                }
            }
        }
        tm_int += 1;
        current[0] += sign_i;
        voxels.push(SingleID::new(
            z,
            current[3 - max_flag],
            current[3 - other_flag_2] as u64,
            current[3 - other_flag_1] as u64,
        )?);
        if tm_int > max_steps {
            print!("WARNING:無限ループを検知!");
            break;
        }
    }
    let iter = voxels.into_iter();
    Ok(iter)
}
