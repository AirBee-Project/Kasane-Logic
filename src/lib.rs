/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
pub mod geometry;
/// 空間IDに関する型を定義。
pub mod spatial_id;

pub use error::Error;

/// よく使うトレイト群をまとめた prelude。
pub mod prelude {
    pub use crate::geometry::point::traits::Point;
    pub use crate::geometry::shapes::traits::{
        IntoCoordinates, IntoLines, IntoPolygons, IntoTriangles, Shape,
    };
    pub use crate::spatial_id::collection::traits::{SpatialIdSet, SpatialIdTable};
    pub use crate::spatial_id::traits::{
        IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialId,
    };
}

// クレート内部実装は既存の `crate::TypeName` 参照を維持する。
#[doc(hidden)]
pub use geometry::constants::{WGS84_A, WGS84_B, WGS84_E2, WGS84_F, WGS84_INV_F};
#[doc(hidden)]
pub use geometry::point::{coordinate::Coordinate, ecef::Ecef, traits::Point};
#[doc(hidden)]
pub use geometry::shapes::{
    line::Line,
    polygon::Polygon,
    solid::Solid,
    sphere::Sphere,
    traits::{IntoCoordinates, IntoLines, IntoPolygons, IntoTriangles, Shape},
    triangle::Triangle,
};
#[doc(hidden)]
pub use spatial_id::collection::flex_tree::{
    core::FlexTreeCore, map::FlexTreeMap, set::FlexTreeSet,
};
#[doc(hidden)]
pub use spatial_id::collection::traits::{SpatialIdSet, SpatialIdTable};
#[doc(hidden)]
pub use spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};
#[doc(hidden)]
pub use spatial_id::flex_id::FlexId;
#[doc(hidden)]
pub use spatial_id::flex_id::segment::Segment;
#[doc(hidden)]
pub use spatial_id::helpers::{Dimension, Side};
#[doc(hidden)]
pub use spatial_id::range_id::RangeId;
#[doc(hidden)]
pub use spatial_id::single_id::SingleId;
#[doc(hidden)]
pub use spatial_id::temporal_id::TemporalId;
#[doc(hidden)]
pub use spatial_id::traits::{IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialId};
