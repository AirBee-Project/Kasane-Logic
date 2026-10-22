use crate::Coordinate;
pub mod geometry_relation;
pub mod impls;
#[cfg(test)]
mod tests;

/// 直線を表す型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    points: [Coordinate; 2],
}

impl Line {
    ///[Line]を作成する。
    pub fn new(points: [Coordinate; 2]) -> Self {
        Self { points }
    }

    /// [Line]の両端の点を返す。
    pub fn points(&self) -> &[Coordinate; 2] {
        &self.points
    }
}
