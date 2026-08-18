#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::disallowed_methods)]
#![deny(clippy::perf)]
#![deny(clippy::needless_collect)]
#[macro_use]
extern crate alloc;

#[cfg(test)]
extern crate std;

/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// `tracing` feature 用の実装
mod trace;

/// 空間ID以外の地理空間情報。
pub mod geometry;
/// 空間IDに関する型を定義。
pub mod spatial_id;

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

// spatial_id: collection types

#[doc(inline)]
pub use spatial_id::collection::flex_tree::set::SpatialIdSet;
#[doc(inline)]
pub use spatial_id::collection::flex_tree::traits::FlexIdValue;

#[doc(inline)]
pub use spatial_id::collection::flex_tree::map::SpatialIdMap;
#[cfg(feature = "persist")]
#[doc(inline)]
pub use spatial_id::collection::flex_tree::map::archived::ArchivedSpatialIdMap;
#[cfg(feature = "persist")]
#[doc(inline)]
pub use spatial_id::collection::flex_tree::map::arena::FORMAT_VERSION;
#[doc(inline)]
pub use spatial_id::collection::flex_tree::table::SpatialIdTable;

// spatial_id: traits
#[doc(inline)]
pub use spatial_id::helpers::Side;
#[doc(inline)]
pub use spatial_id::traits::SpatialId;

// spatial_id: zoom level
#[doc(inline)]
pub use spatial_id::zoom_level::TZoomLevel;
#[doc(inline)]
pub use spatial_id::zoom_level::ZoomLevel;

// spatial_id: query & merge policies
#[doc(inline)]
pub use spatial_id::collection::flex_tree::core::SafeValue;
#[doc(inline)]
pub use spatial_id::collection::query::cancellation::CancellationToken;
#[doc(inline)]
pub use spatial_id::collection::query::execution::Query;
#[doc(inline)]
pub use spatial_id::collection::query::merge_policy;
#[doc(inline)]
pub use spatial_id::collection::query::merge_policy::MergePolicy;
#[doc(inline)]
pub use spatial_id::collection::query::source::Source;
#[doc(inline)]
pub use spatial_id::collection::query::working::WorkingTree;

#[doc(inline)]
pub use spatial_id::time::AllowedIntervals;
#[doc(inline)]
pub use spatial_id::time::Interval;
