#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::disallowed_methods)]
#![warn(clippy::perf)]
#[macro_use]
extern crate alloc;

/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
pub mod geometry;
/// 空間IDに関する型を定義。
pub mod spatial_id;

// `temporal_id` feature を有効にすると、時空間IDの公開APIを時間対応で使えます。
#[doc(inline)]
pub use error::Error;
#[doc(inline)]
pub use geometry::point::{
    coordinate::Coordinate, ecef::Ecef, fractionalid::FractionalId, traits::Point,
};

#[doc(inline)]
pub use error::{GeometryError, SpatialIdError};
#[doc(inline)]
pub use geometry::shape::cylinder::Cylinder;
#[doc(inline)]
pub use geometry::shape::tube::Tube;

// geometry: types
#[doc(inline)]
pub use geometry::shape::line::Line;
#[doc(inline)]
pub use geometry::shape::polygon::Polygon;
#[doc(inline)]
pub use geometry::shape::solid::Solid;
#[doc(inline)]
pub use geometry::shape::sphere::Sphere;

// geometry: vec3 types
#[doc(inline)]
pub use geometry::vec3::vec3_ecef::Vec3Ecef;
#[doc(inline)]
pub use geometry::vec3::vec3_fractionalid::Vec3FractionalId;

// geometry: vec3 traits
#[doc(inline)]
pub use geometry::vec3::traits::Vec3;

// geometry: traits
#[doc(inline)]
pub use geometry::shape::traits::{
    ExpandCoordinates, ExpandLines, ExpandPolygons, ExpandTriangles, Shape,
};
#[doc(inline)]
pub use geometry::shape::triangle::Triangle;
#[doc(inline)]
pub use geometry::traits::{CoverRangeIds, CoverSingleIds};

// geometry: constants
#[doc(inline)]
pub use geometry::constants::{WGS84_A, WGS84_E2, WGS84_F};

// spatial_id: id types
#[doc(inline)]
pub use spatial_id::flex_id::FlexId;
#[doc(inline)]
pub use spatial_id::range_id::RangeId;
#[doc(inline)]
pub use spatial_id::single_id::SingleId;
#[doc(inline)]
pub use spatial_id::temporal_id::Interval;
#[doc(inline)]
pub use spatial_id::temporal_id::TemporalId;
#[doc(inline)]
pub use spatial_id::temporal_id::TemporalMap;
#[doc(inline)]
pub use spatial_id::temporal_id::TemporalSet;

// spatial_id: collection types
#[doc(inline)]
pub(crate) use spatial_id::collection::flex_tree::FlexTree;
#[doc(inline)]
pub use spatial_id::collection::set::SpatialIdSet;
#[doc(inline)]
pub use spatial_id::collection::traits::{CellValue, SpatialIdCollection};

#[doc(inline)]
pub use spatial_id::collection::map::SpatialIdMap;
#[cfg(feature = "persist")]
#[doc(inline)]
pub use spatial_id::collection::map::persist::ArchivedMap;
#[doc(inline)]
pub use spatial_id::collection::table::SpatialIdTable;

// spatial_id: traits
#[doc(inline)]
pub use spatial_id::helpers::{Dimension, Side};
#[doc(inline)]
pub use spatial_id::traits::{IterFlexIds, IterSingleIds, SpatialId};

// spatial_id: zoom level
#[doc(inline)]
pub use spatial_id::zoom_level::ZoomLevel;

#[doc(inline)]
pub use spatial_id::collection::query::Query;
#[doc(inline)]
pub use spatial_id::collection::query::traits::{BinaryOperator, ConflictPolicy, UnaryOperator};
