use alloc::vec::Vec;

pub mod impls;
#[cfg(test)]
mod tests;

use crate::{Coordinate, Error, GeometryError};

#[derive(Debug, Clone, PartialEq)]
pub struct Tube {
    pub points: Vec<Coordinate>,
    pub radius_m: f64,
}
/// 3次元空間におけるパイプを表す型。
impl Tube {
    pub fn new(points: Vec<Coordinate>, radius_m: f64) -> Result<Self, Error> {
        if radius_m > 0.0 {
            Ok(Tube { points, radius_m })
        } else {
            Err(GeometryError::RadiusNegative { radius: radius_m }.into())
        }
    }
}
