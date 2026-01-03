use std::collections::HashSet;

use crate::{
    error::Error,
    geometry::{coordinate::Coordinate, ecef::Ecef},
    id::space_id::single::SingleID,
};


trait RoundTo {
    fn round_to(self, digits: u32) -> f64;
}

impl RoundTo for f64 {
    fn round_to(self, digits: u32) -> f64 {
        let factor = 10f64.powi(digits as i32);
        (self * factor).round() / factor
    }
}


pub fn difference_between_two_point(p1: &Coordinate, p2: &Coordinate) -> f64 {
    let e1: Ecef = (*p1).into();
    let e2: Ecef = (*p2).into();

    ((e1.x - e2.x).powi(2)
        + (e1.y - e2.y).powi(2)
        + (e1.z - e2.z).powi(2))
        .sqrt()
}

pub fn get_length_of_a_voxel(z: u8, axis: &str) -> f64 {
    let vl_xy = (40075016.68 / 2f64.powi(z as i32)).round_to(2);
    let vl_f = (2f64.powi(25 - z as i32)).round_to(2);

    match axis {
        "x" | "y" => vl_xy,
        "f" => vl_f,
        _ => panic!("invalid axis: {}", axis),
    }
}

pub fn center_of_voxel_to_point(voxel: &SingleID) -> Result<(u8, Coordinate), Error> {
    let z = voxel.z;

    let f = voxel.f as f64 + 0.5;
    let x = voxel.x as f64 + 0.5;
    let y = voxel.y as f64 + 0.5;

    let n = 2f64.powi(z as i32);
    let h = 2f64.powi(25);

    let longitude = 180.0 * (2.0 * x / n - 1.0);

    let lat_rad = 2.0
        * ((1.0
            - 2.0
                / (((1.0 - 2.0 * y / n) * std::f64::consts::PI).exp() + 1.0))
            .atan());

    let latitude = lat_rad.to_degrees();
    let altitude = h * f / n;

    let coordinate = Coordinate::new(latitude, longitude, altitude)?;

    Ok((z, coordinate))
}

pub fn get_voxels_inside_sphere(
    z: u8,
    point: &Coordinate,
    mr: f64,
) -> Result<HashSet<SingleID>, Error> {
    let mut voxel_ids = HashSet::new();

    let l = get_length_of_a_voxel(z, "x");
    let epsilon = l * 3.0_f64.sqrt() / 2.0;

    let ecef_center: Ecef = (*point).into();

    let mut corner_voxels = Vec::new();

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let dx = if i == 0 { mr } else { -mr };
                let dy = if j == 0 { mr } else { -mr };
                let dz = if k == 0 { mr } else { -mr };

                let ecef = Ecef::new(
                    ecef_center.x + dx,
                    ecef_center.y + dy,
                    ecef_center.z + dz,
                );

                corner_voxels.push(ecef.to_id(z)?);
            }
        }
    }

    let x_min = corner_voxels.iter().map(|v| v.x).min().unwrap();
    let x_max = corner_voxels.iter().map(|v| v.x).max().unwrap();
    let y_min = corner_voxels.iter().map(|v| v.y).min().unwrap();
    let y_max = corner_voxels.iter().map(|v| v.y).max().unwrap();
    let f_min = corner_voxels.iter().map(|v| v.f).min().unwrap();
    let f_max = corner_voxels.iter().map(|v| v.f).max().unwrap();

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            for f in f_min..=f_max {
                let voxel = SingleID::new(z, f, x, y)?;

                let (_, center) = center_of_voxel_to_point(&voxel)?;

                if difference_between_two_point(point, &center) <= mr + epsilon {
                    voxel_ids.insert(voxel);
                }
            }
        }
    }

    Ok(voxel_ids)
}
