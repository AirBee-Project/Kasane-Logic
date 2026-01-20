/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
mod geometry;

/// 空間IDに関する型を定義。
pub mod spatial_id;

/// このライブライがサポートするストレージのTrait定義と実装
mod storage;

pub use roaring::RoaringTreemap;

pub use error::Error;
pub use geometry::{
    coordinate::Coordinate, ecef::Ecef, shapes::line, shapes::sphere, shapes::triangle,
};
pub use spatial_id::{SpatialId, range_id::RangeId, single_id::SingleId};
pub use storage::BTreeMapTrait;
