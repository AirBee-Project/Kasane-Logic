//! 幾何学図形および地理空間座標を扱うための型やロジック。
//!
//! 本モジュールでは、距離、半径、高さなどの長さに関する値を、
//! 特に明記しない限りメートル（m）単位で扱います。

/// 地理空間座標の計算で使用される各種の代表的な定数。
pub mod constants;

/// 緯度・経度・高度で定義される `Coordinate` 型。
pub mod coordinate;

/// 地心直交座標系で定義される `Ecef` 型。
pub mod ecef;

/// 線分、三角形、円などの幾何形状から空間IDへの変換。
pub mod shapes;

pub mod polygon;

pub mod solid;

pub mod triangle;

pub mod helpers;
