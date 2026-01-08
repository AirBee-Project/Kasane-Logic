use crate::{
    geometry::{coordinate::Coordinate, ecef::Ecef},
    spatial_id::{SpatialId, helpers::Dimension, single::SingleId},
};
pub fn get_length_of_voxel(v: SingleId, a: Dimension) -> f64 {
    match a {
        Dimension::F => 2_i32.pow(25 - v.as_z() as u32) as f64,
        _ => {
            let ecef: Ecef = v.center().into();
            let r = (ecef.as_x() * ecef.as_x() + ecef.as_y() * ecef.as_y()).sqrt();
            r * 2.0 * std::f64::consts::PI / (2_i32.pow(v.as_z() as u32) as f64)
        }
    }
}
pub fn difference(a: Coordinate, b: Coordinate) -> f64 {
    let ecef_a: Ecef = a.into();
    let ecef_b: Ecef = b.into();
    let x = ecef_a.as_x() - ecef_b.as_x();
    let y = ecef_a.as_y() - ecef_b.as_y();
    let z = ecef_a.as_z() - ecef_b.as_z();
    (x * x + y * y + z * z).sqrt()
}
