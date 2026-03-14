use crate::Coordinate;
pub mod geometry_relation;
pub mod impls;

///直線を表す型
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub points: [Coordinate; 2],
}

impl Line {
    ///[Line]を作成する。
    pub fn new(points: [Coordinate; 2]) -> Self {
        Self { points }
    }
}
