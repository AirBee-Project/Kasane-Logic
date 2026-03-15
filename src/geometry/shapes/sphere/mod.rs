use crate::Coordinate;
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
