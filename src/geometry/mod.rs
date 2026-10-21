//! 幾何学図形および地理空間座標を扱うための型やロジック。
//!
//! 本モジュールでは、距離、半径、高さなどの長さに関する値を、
//! 特に明記しない限りメートル（m）単位で扱います。

/// 地理空間座標の計算で使用される各種の代表的な定数。
pub(crate) mod constants;

pub(crate) mod point;

pub(crate) mod shape;

pub(crate) mod traits;

pub(crate) mod vec3;

pub(crate) mod geodata_formats;

pub use geodata_formats::regionmesh::RegionMesh;
pub use point::{coordinate::Coordinate, ecef::Ecef, fractionalid::FractionalId, traits::Point};
pub use shape::traits::{ExpandCoordinates, ExpandLines, ExpandPolygons, ExpandTriangles, Shape};
pub use shape::{
    cylinder::Cylinder, line::Line, polygon::Polygon, solid::Solid, sphere::Sphere,
    triangle::Triangle, tube::Tube,
};
pub use traits::{CoverRangeIds, CoverSingleIds};
