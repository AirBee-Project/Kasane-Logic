use std::f64::consts::PI;

// src/id/space_id/helpers.rs
use crate::error::Error;

/// Scale an inclusive range `[start, end]` by `scale` for children calculation.
/// For integer types, result is `[start*scale, end*scale + scale - 1]`
pub fn scale_range_i64(start: i64, end: i64, scale: i64) -> [i64; 2] {
    [
        start.saturating_mul(scale),
        end.saturating_mul(scale).saturating_add(scale - 1),
    ]
}

pub fn scale_range_u64(start: u64, end: u64, scale: u64) -> [u64; 2] {
    [
        start.saturating_mul(scale),
        end.saturating_mul(scale).saturating_add(scale - 1),
    ]
}

/// n = 2^z を返す
fn n_from_z(z: u8) -> f64 {
    2_f64.powi(z as i32)
}

/// 経度 (longitude) を返す（実数 x 対応）
///
/// x: 水平方向のタイル/セル座標（連続値）  
/// z: ズームレベル  
///
/// セル番号 x の左端なら x、中心なら x+0.5 を渡せる。
pub fn longitude(x: f64, z: u8) -> f64 {
    let n = n_from_z(z);
    360.0 * (x / n) - 180.0
}

/// 緯度 (latitude) を返す（Web Mercator の逆変換, 実数 y 対応）
///
/// y: 垂直方向のタイル/セル座標（連続値）  
/// z: ズームレベル  
///
/// 公式: lat = atan( sinh( π * (1 - 2*y/n) ) )
pub fn latitude(y: f64, z: u8) -> f64 {
    let n = n_from_z(z);
    let t = PI * (1.0 - 2.0 * (y / n));
    let lat_rad = t.sinh().atan();
    lat_rad.to_degrees()
}

/// 高度 (altitude) を返す（実数 f 対応）
///
/// f: 高度方向 index（連続値）  
/// z: ズームレベル  
///
pub fn altitude(f: f64, z: u8) -> f64 {
    let n = n_from_z(z);
    33_554_432.0 * (f / n)
}
