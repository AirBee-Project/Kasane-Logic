/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
pub mod geometry;
/// 空間IDに関する型を定義。
pub mod spatial_id;

// `temporal_id` feature を有効にすると、時空間IDの公開APIを時間対応で使えます。
#[doc(inline)]
pub use error::Error;
pub use geometry::point::{coordinate::Coordinate, ecef::Ecef, traits::Point};

#[doc(inline)]
pub use error::{GeometryError, SpatialIdError};
pub use geometry::constants::*;
pub use geometry::shapes::cylinder::Cylinder;

// geometry: types
#[doc(inline)]
pub use geometry::point::coordinate::Coordinate;
#[doc(inline)]
pub use geometry::point::ecef::Ecef;
#[doc(inline)]
pub use geometry::shapes::line::Line;
#[doc(inline)]
pub use geometry::shapes::polygon::Polygon;
#[doc(inline)]
pub use geometry::shapes::solid::Solid;
#[doc(inline)]
pub use geometry::shapes::sphere::Sphere;
#[doc(inline)]
pub use geometry::shapes::triangle::Triangle;

// geometry: traits
#[doc(inline)]
pub use geometry::point::traits::Point;
#[doc(inline)]
pub use geometry::shapes::traits::{
    IntoCoordinates, IntoLines, IntoPolygons, IntoTriangles, Shape,
};
#[doc(inline)]
pub use geometry::traits::{CoverRangeIds, CoverSingleIds};

// geometry: constants
#[doc(inline)]
pub use geometry::constants::{WGS84_A, WGS84_B, WGS84_E2, WGS84_F, WGS84_INV_F};

// spatial_id: id types
#[doc(inline)]
pub use spatial_id::flex_id::FlexId;
#[doc(inline)]
pub use spatial_id::range_id::RangeId;
#[doc(inline)]
pub use spatial_id::single_id::SingleId;
#[doc(inline)]
pub use spatial_id::temporal_id::TemporalId;

// spatial_id: collection types
#[doc(inline)]
pub(crate) use spatial_id::collection::flex_tree::core::FlexTreeCore;
#[doc(inline)]
pub use spatial_id::collection::flex_tree::map::SpatialIdMap;
#[doc(inline)]
pub use spatial_id::collection::flex_tree::set::SpatialIdSet;

#[doc(inline)]
pub use spatial_id::collection::flex_tree::table::SpatialIdTable;

// spatial_id: traits
#[doc(inline)]
pub use spatial_id::helpers::{Dimension, Side};
#[doc(inline)]
pub use spatial_id::traits::{IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialId};

// spatial_id: constants
#[doc(inline)]
pub use spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};
