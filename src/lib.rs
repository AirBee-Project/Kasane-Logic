/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
mod geometry;

/// 空間IDに関する型を定義。
mod spatial_id;

pub use roaring::RoaringTreemap;

pub use error::Error;
pub use geometry::point::{Point, coordinate::Coordinate, ecef::Ecef};

pub use geometry::shapes::polygon::Polygon;
pub use geometry::shapes::solid::Solid;
pub use geometry::shapes::triangle;
pub use spatial_id::Block;
pub use spatial_id::SpatialId;
pub use spatial_id::collection::FlexIdRank;
pub use spatial_id::collection::set::SetOnMemory;
pub use spatial_id::collection::table::TableOnMemory;
pub use spatial_id::collection::traits::SpatialIdSet;
pub use spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};
pub use spatial_id::flex_id::FlexId;
pub use spatial_id::flex_id::segment::Segment;
pub use spatial_id::helpers::fast_intersect;
pub use spatial_id::range_id::RangeId;
pub use spatial_id::single_id::SingleId;
pub use spatial_id::temporal_id::TemporalId;
