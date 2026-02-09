/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
mod geometry;

/// 空間IDに関する型を定義。
mod spatial_id;

pub use roaring::RoaringTreemap;

pub use error::Error;
pub use geometry::{
    coordinate::Coordinate, ecef::Ecef, shapes::line::line, shapes::sphere::sphere,
    shapes::triangle::triangle,
};
pub use spatial_id::{
    SpatialId,
    constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
    flex_id::FlexId,
    range_id::RangeId,
    single_id::SingleId,
};

pub use geometry::solid::Solid;
pub use geometry::surface::Surface;
pub use spatial_id::collection::FlexIdRank;
pub use spatial_id::collection::set::SetOnMemory;
pub use spatial_id::segment::Segment;
