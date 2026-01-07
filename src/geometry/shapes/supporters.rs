use crate::{
    geometry::ecef::Ecef,
    spatial_id::{SpatialId, single::SingleId},
};
pub enum Axis {
    F,
    X,
    Y,
}
pub fn get_length_of_voxel(v: SingleId, a: Axis) -> f64 {
    match a {
        Axis::F => 2_i32.pow(25 - v.as_z() as u32) as f64,
        _ => {
            let ecef: Ecef = v.center().into();
            let r = (ecef.as_x() * ecef.as_x() + ecef.as_y() * ecef.as_y()).sqrt();
            r * 2.0 * std::f64::consts::PI / (2_i32.pow(v.as_z() as u32) as f64)
        }
    }
}
