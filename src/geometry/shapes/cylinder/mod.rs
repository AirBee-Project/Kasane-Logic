pub mod geometry_relation;
pub mod impls;

use crate::{Coordinate, Error, GeometryError};
#[derive(Debug, Clone, Copy, PartialEq)]
/// 3次元空間における円柱を表す型。
///
/// 中心線及び半径によって定義される立体的な領域を表現する。
pub struct Cylinder {
    pub start: Coordinate,
    pub end: Coordinate, //向きに明確な意味を持たせるならstart,endを別に定義する方がいい？
    pub radius_m: f64,
}

impl Cylinder {
    pub fn new(start: Coordinate, end: Coordinate, radius_m: f64) -> Result<Self, Error> {
        if radius_m > 0.0 {
            Ok(Cylinder {
                start,
                end,
                radius_m,
            })
        } else {
            Err(GeometryError::RadiusNegative { radius: radius_m }.into())
        }
    }
}
