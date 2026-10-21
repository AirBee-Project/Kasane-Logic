#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::disallowed_methods)]
#![deny(clippy::perf)]
#![deny(clippy::needless_collect)]
#[macro_use]
extern crate alloc;

#[cfg(test)]
extern crate std;

// derive マクロが生成する `::kasane_logic::…` を、このクレートの中でも解決できるようにする。
extern crate self as kasane_logic;

/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間IDの集合と、空間IDごとに値を持つマップ。
pub mod collection;
/// 空間ID以外の地理空間情報。
pub mod geometry;
pub mod prelude;
/// 空間IDのデータを組み合わせて変換するクエリ。
pub mod query;
/// 空間IDに関する型を定義。
pub mod spatial_id;

pub use error::{Error, GeometryError, SpatialIdError};

// クレート内部では、公開パスを書かずに短く参照する
#[allow(unused_imports)]
pub(crate) use collection::summary::{BitMask, MinMax, NoSummary, Summary, ValueSet};
#[allow(unused_imports)]
pub(crate) use collection::{SpatialIdMap, SpatialIdSet};
#[allow(unused_imports)]
pub(crate) use geometry::constants::{WGS84_A, WGS84_E2, WGS84_F};
#[allow(unused_imports)]
pub(crate) use geometry::vec3::{
    traits::Vec3, vec3_ecef::Vec3Ecef, vec3_fractionalid::Vec3FractionalId,
};
#[allow(unused_imports)]
pub(crate) use geometry::*;
#[allow(unused_imports)]
pub(crate) use query::{
    CancellationToken, IntoTargets, Query, QueryOutput, SafeValue, Source, merge_policy,
    merge_policy::MergePolicy,
};
#[allow(unused_imports)]
pub(crate) use spatial_id::relative_flex_id::RelativeFlexId;
#[allow(unused_imports)]
pub(crate) use spatial_id::*;
