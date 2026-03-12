//! Kasane-Logic は、解析解を用いた図形から空間IDへの高精度な変換アルゴリズムを
//! Rust で実装したコアライブラリです。
//!
//! GitHub 上で MIT ライセンスとして公開しており、Rust の型システムを活かした
//! 空間ID関連の豊富な型定義を提供します。
//! 第三者が空間ID関連アプリケーションを実装する際に、そのまま土台として活用できます。
//!
//! # 主な使い方
//! ## 1) 座標から `SingleId` を作る
//! ```
//! use kasane_logic::{Coordinate, SingleId};
//!
//! let coord = Coordinate::new(35.681236, 139.767125, 12.0).unwrap();
//! let id: SingleId = coord.to_single_id(18).unwrap();
//!
//! assert_eq!(id.z(), 18);
//! ```
//!
//! ## 2) `RangeId` で領域を表現して `SingleId` 列に展開する
//! ```
//! use kasane_logic::{RangeId, SpatialId};
//!
//! let range = RangeId::new(5, [-1, 1], [2, 3], [4, 4]).unwrap();
//! let cells: Vec<_> = range.single_ids().collect();
//!
//! assert_eq!(cells.len(), 6);
//! ```
//!
//! ## 3) 閉じた立体 (`Solid`) を空間IDへ変換する
//! ```
//! use kasane_logic::{Coordinate, Solid};
//!
//! let a = Coordinate::new(35.0, 139.0, 0.0).unwrap();
//! let b = Coordinate::new(35.0, 139.001, 0.0).unwrap();
//! let c = Coordinate::new(35.001, 139.0, 0.0).unwrap();
//! let d = Coordinate::new(35.0, 139.0, 20.0).unwrap();
//!
//! let surfaces = vec![
//!     vec![a, b, c],
//!     vec![a, b, d],
//!     vec![b, c, d],
//!     vec![c, a, d],
//! ];
//!
//! let solid = Solid::new(surfaces, 0.01).unwrap();
//! let voxels: Vec<_> = solid.single_ids(18).unwrap().collect();
//!
//! assert!(!voxels.is_empty());
//! ```

/// 発生し得るすべてのエラーを`enum` 型として定義・集約。
mod error;

/// 空間ID以外の地理空間情報。
pub mod geometry;

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
// pub use spatial_id::collection::SpatialIdSet;
pub use spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX};
pub use spatial_id::flex_id::FlexId;
pub use spatial_id::flex_id::segment::Segment;
pub use spatial_id::helpers::fast_intersect;
pub use spatial_id::range_id::RangeId;
pub use spatial_id::single_id::SingleId;
pub use spatial_id::temporal_id::TemporalId;
