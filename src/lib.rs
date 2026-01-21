/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
mod geometry;

/// 空間IDに関する型を定義。
mod spatial_id;

/// このライブライがサポートするストレージのTrait定義と実装
mod storage;

pub use roaring::RoaringTreemap;

pub use error::Error;
pub use geometry::{
    coordinate::Coordinate, ecef::Ecef, shapes::line, shapes::sphere, shapes::triangle,
};
pub use spatial_id::{
    SpatialId,
    constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
    range_id::RangeId,
    single_id::SingleId,
};

pub use spatial_id::collection::set::{SetStorage, logic::SetLogic, memory::SetOnMemory};

pub use storage::{KeyValueStore, OrderedKeyValueStore};
