//! 空間IDの集合と、空間IDごとに値を持つマップ。

pub(crate) mod core;
#[cfg(feature = "json")]
mod json;
mod map;
mod set;
pub mod summary;
pub(crate) mod time_reconstruct;
mod traits;

pub use self::core::IntoIter;
pub use map::SpatialIdMap;
pub use set::SpatialIdSet;
