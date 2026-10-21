//! メソッドを呼ぶのに必要なTraitをまとめて import する。
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
