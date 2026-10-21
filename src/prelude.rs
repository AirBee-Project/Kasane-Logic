//! メソッドを呼ぶために import が必要なTraitをまとめたモジュール。
//!
//! ```
//! use kasane_logic::prelude::*;
//! ```

pub use crate::geometry::{
    CoverRangeIds, CoverSingleIds, ExpandCoordinates, ExpandLines, ExpandPolygons, ExpandTriangles,
    Point, Shape,
};
pub use crate::query::Source;
pub use crate::spatial_id::SpatialId;
