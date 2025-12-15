use std::{cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{
    geometry::{constants::WGS84_A, coordinate::Coordinate, ecef::Ecef},
    id::space_id::single::SingleID,
};

pub fn triangle(
    z: u8,
    a: Coordinate,
    b: Coordinate,
    c: Coordinate,
) -> impl Iterator<Item = SingleID> {
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();
    let ecef_c: Ecef = c.into();

    let min_lat_rad = a
        .latitude
        .abs()
        .min(b.latitude.abs())
        .min(c.latitude.abs())
        .to_radians();
    let d = PI * WGS84_A * min_lat_rad.cos() * 2f64.powi(-2 - z as i32);

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

    let seen = Rc::new(RefCell::new(HashSet::new()));

    (0..=steps).flat_map(move |i| {
        let t = i as f64 / steps as f64;

        let line1 = (
            ecef_a.x * (1.0 - t) + ecef_b.x * t,
            ecef_a.y * (1.0 - t) + ecef_b.y * t,
            ecef_a.z * (1.0 - t) + ecef_b.z * t,
        );
        let line2 = (
            ecef_a.x * (1.0 - t) + ecef_c.x * t,
            ecef_a.y * (1.0 - t) + ecef_c.y * t,
            ecef_a.z * (1.0 - t) + ecef_c.z * t,
        );

        let seen = seen.clone(); // クロージャごとに clone

        (0..=i).filter_map(move |j| {
            let (x, y, z_pos) = if i == 0 {
                (ecef_a.x, ecef_a.y, ecef_a.z)
            } else {
                let s = j as f64 / i as f64;
                (
                    line1.0 * (1.0 - s) + line2.0 * s,
                    line1.1 * (1.0 - s) + line2.1 * s,
                    line1.2 * (1.0 - s) + line2.2 * s,
                )
            };

            if let Ok(voxel_id) = Ecef::new(x, y, z_pos).to_id(z) {
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
    })
}
