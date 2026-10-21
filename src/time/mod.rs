//! 時間の区間と、時間のズームレベルを表現する。

pub(crate) mod allowed_intervals;
pub(crate) mod interval;
pub(crate) mod segment;
pub(crate) mod span;

pub use crate::spatial_id::zoom_level::TZoomLevel;
pub use allowed_intervals::AllowedIntervals;
pub use interval::Interval;
