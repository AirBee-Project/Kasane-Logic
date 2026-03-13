use crate::{
    Coordinate, Ecef, SingleId, SpatialId, geometry::constants::WGS84_A,
    spatial_id::helpers::Dimension,
};
pub mod impls;

///球体を表す型
pub struct Sphere {
    center: Coordinate,
    radius_m: f64,
}

impl Sphere {
    ///[Sphere]を作成する。
    pub fn new(center: Coordinate, radius_m: f64) -> Self {
        Sphere { center, radius_m }
    }
}
