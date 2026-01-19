use std::{cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{
    error::Error,
    geometry::{constants::WGS84_A, coordinate::Coordinate, ecef::Ecef},
    spatial_id::{constants::MAX_ZOOM_LEVEL, single_id::SingleId},
};

/// 指定された 3 点で構成される三角形領域を覆う空間 ID を列挙する。
pub fn triangle(
    z: u8,
    a: Coordinate,
    b: Coordinate,
    c: Coordinate,
) -> Result<impl Iterator<Item = SingleId>, Error> {
    if z > MAX_ZOOM_LEVEL as u8 {
        return Err(Error::ZOutOfRange { z });
    }

    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();
    let ecef_c: Ecef = c.into();

    let min_lat_rad = a
        .as_latitude()
        .abs()
        .min(b.as_latitude().abs())
        .min(c.as_latitude().abs())
        .to_radians();

    let d = PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-2 - z as i32);

    let l1 = ((ecef_c.as_x() - ecef_b.as_x()).powi(2)
        + (ecef_c.as_y() - ecef_b.as_y()).powi(2)
        + (ecef_c.as_z() - ecef_b.as_z()).powi(2))
    .sqrt();
    let l2 = ((ecef_a.as_x() - ecef_c.as_x()).powi(2)
        + (ecef_a.as_y() - ecef_c.as_y()).powi(2)
        + (ecef_a.as_z() - ecef_c.as_z()).powi(2))
    .sqrt();
    let l3 = ((ecef_a.as_x() - ecef_b.as_x()).powi(2)
        + (ecef_a.as_y() - ecef_b.as_y()).powi(2)
        + (ecef_a.as_z() - ecef_b.as_z()).powi(2))
    .sqrt();

    let steps = (l1.max(l2).max(l3) / d).ceil() as usize;

    let seen = Rc::new(RefCell::new(HashSet::new()));

    let iter = (0..=steps).flat_map(move |i| {
        let t = i as f64 / steps as f64;

        let line1 = (
            ecef_a.as_x() * (1.0 - t) + ecef_b.as_x() * t,
            ecef_a.as_y() * (1.0 - t) + ecef_b.as_y() * t,
            ecef_a.as_z() * (1.0 - t) + ecef_b.as_z() * t,
        );
        let line2 = (
            ecef_a.as_x() * (1.0 - t) + ecef_c.as_x() * t,
            ecef_a.as_y() * (1.0 - t) + ecef_c.as_y() * t,
            ecef_a.as_z() * (1.0 - t) + ecef_c.as_z() * t,
        );

        let seen = seen.clone();

        (0..=i).filter_map(move |j| {
            let (x, y, z_pos) = if i == 0 {
                (ecef_a.as_x(), ecef_a.as_y(), ecef_a.as_z())
            } else {
                let s = j as f64 / i as f64;
                (
                    line1.0 * (1.0 - s) + line2.0 * s,
                    line1.1 * (1.0 - s) + line2.1 * s,
                    line1.2 * (1.0 - s) + line2.2 * s,
                )
            };

            if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_single_id(z) {
                let mut borrowed = seen.borrow_mut();
                if borrowed.insert(voxel_id.clone()) {
                    Some(voxel_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    });

    Ok(iter)
}
