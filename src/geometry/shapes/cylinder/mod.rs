use crate::{Coordinate, Error};
#[derive(Debug, Clone)]
/// 3次元空間における円柱を表す型。
///
/// 中心線及び半径によって定義される立体的な領域を表現する。
pub struct Cylinder {
    pub points: [Coordinate; 2], //向きに明確な意味を持たせるならstart,endを別に定義する方がいい？
    pub radius: f64,
}

impl Cylinder {
    pub fn new(points: [Coordinate; 2], radius: f64) -> Result<Self, Error> {
        if radius > 0.0 {
            Ok(Cylinder { points, radius })
        } else {
            Err(Error::RadiusNegative { radius: radius })
        }
    }
}
