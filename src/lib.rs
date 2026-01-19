use std::ops::{BitAnd, BitOr, Deref, DerefMut, Not, Sub};

use crate::spatial_id::{
    SpatialIdEncode,
    collection::{
        map::{MapLogic, OnMemoryMap},
        set::OnMemorySet,
    },
};

/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
mod geometry;

/// 空間IDに関する型を定義。
mod spatial_id;

/// このライブライがサポートするストレージのTrait定義と実装
pub mod kv;

pub use error::Error;
pub use geometry::{coordinate::Coordinate, ecef::Ecef};
pub use spatial_id::{range_id::RangeId, single_id::SingleId};

#[derive(Clone)]
pub struct SpatialIdMap<V>(MapLogic<OnMemoryMap<V>>);
impl<V> SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    pub fn new() -> Self {
        Self(MapLogic::new(OnMemoryMap::default()))
    }
}

impl<V> Default for SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Deref for SpatialIdMap<V> {
    type Target = MapLogic<OnMemoryMap<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for SpatialIdMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Default)]
pub struct SpatialIdSet(OnMemorySet);

impl SpatialIdSet {
    pub fn new() -> Self {
        Self(OnMemorySet::new())
    }
}

impl Deref for SpatialIdSet {
    type Target = OnMemorySet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpatialIdSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
