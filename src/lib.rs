/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
pub mod geometry;
/// 空間IDに関する型を定義。
pub mod spatial_id;

pub use error::Error;

// geometry
pub use geometry::constants::{WGS84_A, WGS84_B, WGS84_E2, WGS84_F, WGS84_INV_F};
pub use geometry::point::coordinate::Coordinate;
pub use geometry::point::ecef::Ecef;
pub use geometry::point::traits::Point;
pub use geometry::shapes::line::Line;
pub use geometry::shapes::polygon::Polygon;
pub use geometry::shapes::solid::Solid;
pub use geometry::shapes::sphere::Sphere;
pub use geometry::shapes::traits::{
    IntoCoordinates, IntoLines, IntoPolygons, IntoTriangles, Shape,
};
pub use geometry::shapes::triangle::Triangle;
pub use geometry::traits::{ToFlexIds, ToRangeIds, ToSingleIds};

// spatial_id
pub use spatial_id::collection::flex_tree::core::FlexTreeCore;
pub use spatial_id::collection::flex_tree::map::FlexTreeMap;
pub use spatial_id::collection::flex_tree::set::FlexTreeSet;
pub use spatial_id::collection::traits::{SpatialIdSet, SpatialIdTable};
pub use spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};
pub use spatial_id::flex_id::FlexId;
pub use spatial_id::flex_id::segment::Segment;
pub use spatial_id::helpers::{Dimension, Side};
pub use spatial_id::range_id::RangeId;
pub use spatial_id::single_id::SingleId;
pub use spatial_id::temporal_id::TemporalId;
pub use spatial_id::traits::{IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialId};
