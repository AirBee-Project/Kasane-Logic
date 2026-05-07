use crate::{Coordinate, Ecef};
pub mod geometry_relation;
pub mod impls;

///直線を表す型
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub points: [Coordinate; 2],
}

impl Line {
    ///[Line]を作成する。
    pub fn new(points: [Coordinate; 2]) -> Self {
        Self { points }
    }

    ///重心を求める
    pub fn center(&self) -> Coordinate {
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();

        let center = Ecef::new(
            (p0.x() + p1.x()) / 2.0,
            (p0.y() + p1.y()) / 2.0,
            (p0.z() + p1.z()) / 2.0,
        );

        center
            .try_into()
            .unwrap_or_else(|_| panic!("Failed to convert triangle center"))
    }
}
