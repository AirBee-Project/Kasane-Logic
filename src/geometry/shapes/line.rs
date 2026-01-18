use std::f64;

use crate::{
    error::Error,
    geometry::{coordinate::Coordinate, ecef::Ecef},
    spatial_id::{constants::MAX_ZOOM_LEVEL, single::SingleId},
};

/// 指定された 2 点で構成される直線を覆う空間 ID を列挙する。
pub fn line(z: u8, a: Coordinate, b: Coordinate) -> Result<impl Iterator<Item = SingleId>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();
    let dx = ecef_a.as_x() - ecef_b.as_x();
    let dy = ecef_a.as_y() - ecef_b.as_y();
    let dz = ecef_a.as_z() - ecef_b.as_z();
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    let (v1, v2) = (a.to_single_id(z), b.to_single_id(z));
    let diff = ((v1.as_f() - v2.as_f()).abs()
        + (v1.as_x() as i32 - v2.as_x() as i32).abs()
        + (v1.as_y() as i32 - v2.as_y() as i32).abs()) as f64;
    let devide_num = 5 + (diff / 120.0 + distance / 2000.0).floor() as u16;
    let mut coordinates = Vec::new();
    for i in 0..=devide_num {
        let t = i as f64 / devide_num as f64;
        let x = ecef_a.as_x() * (1.0 - t) + ecef_b.as_x() * t;
        let y = ecef_a.as_y() * (1.0 - t) + ecef_b.as_y() * t;
        let z_pos = ecef_a.as_z() * (1.0 - t) + ecef_b.as_z() * t;
        let coo: Coordinate = Ecef::new(x, y, z_pos).try_into()?;
        coordinates.push(coo);
    }
    let mut voxels: Vec<SingleId> = Vec::new();
    for pair in coordinates.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let line_iter = line_dda(z, start, end)?;
        voxels.pop();
        voxels.extend(line_iter);
    }
    Ok(voxels.into_iter())
}

fn coordinate_to_matrix(p: Coordinate, z: u8) -> [f64; 3] {
    let lat = p.as_latitude();
    let lon = p.as_longitude();
    let alt = p.as_altitude();

    // 空間idの高さはz=25でちょうど1mになるように定義されている
    let factor = 2_f64.powi(z as i32 - 25);
    let f = factor * alt;

    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    [f, x, y]
}

///DDAを用いたLine関数
fn line_dda(z: u8, a: Coordinate, b: Coordinate) -> Result<impl Iterator<Item = SingleId>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }
    let origin1 = coordinate_to_matrix(a, z);
    let origin2 = coordinate_to_matrix(b, z);
    let offsets = origin1.map(|x| x.floor());
    let vp1 = [
        origin1[0] - offsets[0],
        origin1[1] - offsets[1],
        origin1[2] - offsets[2],
    ];
    let vp2 = [
        origin2[0] - offsets[0],
        origin2[1] - offsets[1],
        origin2[2] - offsets[2],
    ];
    let d_total = [
        (vp2[0] - vp1[0]).abs(),
        (vp2[1] - vp1[1]).abs(),
        (vp2[2] - vp1[2]).abs(),
    ];
    let offsets_int = offsets.map(|x| x as i32);
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
    let i1 = vp1[max_flag].floor() as i32;
    let j1 = vp1[other_flag_1].floor() as i32;
    let k1 = vp1[other_flag_2].floor() as i32;
    let i2 = vp2[max_flag].floor() as i32;
    let j2 = vp2[other_flag_1].floor() as i32;
    let k2 = vp2[other_flag_2].floor() as i32;
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
        1.0 - vp1[max_flag] + vp1[max_flag].floor()
    } else if i2 == i1 {
        f64::INFINITY
    } else {
        vp1[max_flag] - vp1[max_flag].floor()
    };
    let mut to1 = if j2 > j1 {
        (1.0 - vp1[other_flag_1] + vp1[other_flag_1].floor()) * d_o1 - tm
    } else if j2 == j1 {
        f64::INFINITY
    } else {
        (vp1[other_flag_1] - vp1[other_flag_1].floor()) * d_o1 - tm
    };
    let mut to2 = if k2 > k1 {
        (1.0 - vp1[other_flag_2] + vp1[other_flag_2].floor()) * d_o2 - tm
    } else if k2 == k1 {
        f64::INFINITY
    } else {
        (vp1[other_flag_2] - vp1[other_flag_2].floor()) * d_o2 - tm
    };
    let max_steps = ((i2 - i1).abs() + (j2 - j1).abs() + (k2 - k1).abs()) as usize;
    let pull_index = [
        (3 - max_flag) % 3,
        (3 - other_flag_2) % 3,
        (3 - other_flag_1) % 3,
    ];
    let mut current = [i1, j1, k1];
    let sign_i = (vp2[max_flag] - vp1[max_flag]).signum() as i32;
    let sign_j = (vp2[other_flag_1] - vp1[other_flag_1]).signum() as i32;
    let sign_k = (vp2[other_flag_2] - vp1[other_flag_2]).signum() as i32;
    let mut tm_int = 0;
    let first = unsafe {
        SingleId::new_unchecked(
            z,
            current[pull_index[0]] + offsets_int[0],
            (current[pull_index[1]] + offsets_int[1]) as u32,
            (current[pull_index[2]] + offsets_int[2]) as u32,
        )
    };
    let iter = std::iter::once(first).chain((1..=max_steps).map(move |_| {
        let min_wall = (tm_int as f64).min(to1).min(to2);
        if min_wall == tm_int as f64 {
            tm_int += 1;
            current[0] += sign_i;
        } else if min_wall == to1 {
            to1 += d_o1;
            current[1] += sign_j;
        } else {
            to2 += d_o2;
            current[2] += sign_k;
        }
        unsafe {
            SingleId::new_unchecked(
                z,
                current[pull_index[0]] + offsets_int[0],
                (current[pull_index[1]] + offsets_int[1]) as u32,
                (current[pull_index[2]] + offsets_int[2]) as u32,
            )
        }
    }));
    Ok(iter)
}
