//! 座標系と地心地固直交座標系（ECEF）を扱うモジュール
//!
//! このモジュールは、WGS-84楕円体上の座標とECEF座標系の相互変換を提供します。
//! 
//! # 主な型
//! - [`Coordinate`]: WGS-84楕円体上の緯度・経度・高度を表現
//! - [`Ecef`]: 地心地固直交座標系（ECEF）の座標を表現

mod coordinate;
mod ecef;

pub use coordinate::Coordinate;
pub use ecef::Ecef;
