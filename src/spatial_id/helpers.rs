use std::f64::consts::PI;

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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
///次元を選択するEnum
/// u8に変換される
pub enum Dimension {
    F,
    X,
    Y,
}

use roaring::RoaringTreemap;

///[RoaringTreemap]が大量にあったときに、最も高速に積集合を求める関数
pub fn fast_intersect<'a, I>(sets: I) -> RoaringTreemap
where
    I: IntoIterator<Item = &'a RoaringTreemap>,
{
    let mut vec: Vec<&RoaringTreemap> = sets.into_iter().collect();

    if vec.is_empty() {
        return RoaringTreemap::new();
    }

    // 空集合があれば即終了
    if vec.iter().any(|s| s.is_empty()) {
        return RoaringTreemap::new();
    }

    // 小さい順に並べる
    vec.sort_by_key(|s| s.len());

    // 最小だけ clone
    let mut result = vec[0].clone();

    // 破壊的 AND
    for s in &vec[1..] {
        result &= *s;
        if result.is_empty() {
            break;
        }
    }

    result
}
