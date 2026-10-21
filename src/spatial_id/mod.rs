//! 空間ID（[`SingleId`]、[`RangeId`]、[`FlexId`]）と、ズームレベルや時間の区間。

pub(crate) mod dimension;
pub(crate) mod single_id;
pub(crate) mod time;
pub(crate) mod traits;
pub(crate) mod zoom_level;

pub(crate) mod flex_id;
pub(crate) mod helpers;
pub(crate) mod range_id;
pub(crate) mod relative_flex_id;

#[cfg(test)]
mod tests;

pub use crate::error::SpatialIdError;
pub(crate) use dimension::Dimension;
pub use flex_id::FlexId;
pub use helpers::{IntoRange, Side};
pub use range_id::RangeId;
pub use single_id::{SingleId, convert::SingleIdSegments};
pub use time::{AllowedIntervals, Interval};
pub use traits::SpatialId;
pub use zoom_level::{TZoomLevel, Zoom, ZoomLevel};
