use std::{f64::consts::PI, fmt};
/// Scale an inclusive range `[start, end]` by `scale` for children calculation.
/// For integer types, result is `[start*scale, end*scale + scale - 1]`
pub fn scale_range_i32(start: i32, end: i32, scale: i32) -> [i32; 2] {
    [
        start.saturating_mul(scale),
        end.saturating_mul(scale).saturating_add(scale - 1),
    ]
}

pub fn scale_range_u32(start: u32, end: u32, scale: u32) -> [u32; 2] {
    [
        start.saturating_mul(scale),
        end.saturating_mul(scale).saturating_add(scale - 1),
    ]
}

/// 経度 (longitude) を返す（実数 x 対応）
///
/// x: 水平方向のタイル/セル座標（連続値）  
/// z: ズームレベル  
///
/// セル番号 x の左端なら x、中心なら x+0.5 を渡せる。
pub fn longitude(x: f64, z: u8) -> f64 {
    let n = 2_f64.powi(z as i32);
    360.0 * (x / n) - 180.0
}

/// 緯度 (latitude) を返す（Web Mercator の逆変換, 実数 y 対応）
///
/// y: 垂直方向のタイル/セル座標（連続値）  
/// z: ズームレベル  
///
/// 公式: lat = atan( sinh( π * (1 - 2*y/n) ) )
pub fn latitude(y: f64, z: u8) -> f64 {
    let n = 2_f64.powi(z as i32);
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
    let n = 2_f64.powi(z as i32);
    33_554_432.0 * (f / n)
}

///次元を選択するEnum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Dimension {
    F = 0,
    X = 1,
    Y = 2,
}

///上下,左右などの2分岐の概念を表すEnum

/// 座標軸で小さい側:[Side::Lower]
/// 座標軸で大きい側:[Side::Upper]
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C)]
pub enum Side {
    /// 座標が小さい方 (Left, Down, Low-F)
    Lower = 0,
    /// 座標が大きい方 (Right, Up, High-F)
    Upper = 1,
}

///次元の区間表記の文字列を圧縮するための関数
pub fn format_dimension<T: PartialEq + fmt::Display>(dimension: [T; 2]) -> String {
    if dimension[0] == dimension[1] {
        format!("{}", dimension[0])
    } else {
        format!("{}:{}", dimension[0], dimension[1])
    }
}
