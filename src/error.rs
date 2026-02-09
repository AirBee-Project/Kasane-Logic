use std::error;
use std::fmt;

/// 本クレートで発生し得るエラーを表す。
#[derive(Debug, PartialEq)] // PartialEqはテスト等で便利ですが、f64を含む場合は注意が必要です
pub enum Error {
    // --- 既存の座標・ズームレベルエラー ---
    /// ズームレベルが有効範囲（0..=31）外であることを示す。
    ZOutOfRange { z: u8 },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して有効範囲外であることを示す。
    FOutOfRange { z: u8, f: i32 },

    /// X 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    XOutOfRange { z: u8, x: u32 },

    /// Y 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    YOutOfRange { z: u8, y: u32 },

    /// 緯度が有効範囲外であることを表す。
    LatitudeOutOfRange { latitude: f64 },

    /// 経度が有効範囲外であることを表す。
    LongitudeOutOfRange { longitude: f64 },

    /// 高度が有効範囲外であることを表す。
    AltitudeOutOfRange { altitude: f64 },

    // --- 新規追加: Surface/Solid 幾何形状検証エラー ---
    /// ポリゴンの頂点数が不足している（閉じるためには最低4点必要）。
    /// 保持する値は実際の頂点数。
    TooFewPoints(usize),

    /// ポリゴンの始点と終点が一致していない（閉じていない）。
    NotClosedRing,

    /// ポリゴンの頂点が一直線上に並んでおり、面を定義できない（共線）。
    /// 法線ベクトルの計算不能時に発生。
    CollinearPoints,

    /// ポリゴンの頂点が同一平面上にない（ねじれている）。
    NotPlanar,

    /// Solid（立体）が空である（面が含まれていない）。
    EmptySolid,

    /// 縮退したエッジ（長さが0の辺）が含まれている。
    /// 保持する値は、そのエッジを含む面のインデックス。
    DegenerateEdge(usize),

    /// 穴が開いている（閉じた立体ではない）。
    /// エッジが片側の面からしか参照されていない場合に発生。
    OpenHoleDetected,

    /// 非多様体エッジ（3つ以上の面が共有）、または面の向きが不整合（法線フリップ）。
    NonManifoldEdge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // --- 既存のエラーメッセージ ---
            Error::ZOutOfRange { z } => {
                write!(f, "ZoomLevel '{}' is out of range (valid: 0..=60)", z)
            }
            Error::FOutOfRange { z, f: fv } => {
                write!(
                    f,
                    "F coordinate '{}' is out of range for ZoomLevel '{}'",
                    fv, z
                )
            }
            Error::XOutOfRange { z, x } => {
                write!(
                    f,
                    "X coordinate '{}' is out of range for ZoomLevel '{}'",
                    x, z
                )
            }
            Error::YOutOfRange { z, y } => {
                write!(
                    f,
                    "Y coordinate '{}' is out of range for ZoomLevel '{}'",
                    y, z
                )
            }
            Error::LatitudeOutOfRange { latitude } => {
                write!(
                    f,
                    "Latitude '{}' is out of range (valid: -85.0511..=85.0511)",
                    latitude
                )
            }
            Error::LongitudeOutOfRange { longitude } => {
                write!(
                    f,
                    "Longitude '{}' is out of range (valid: -180.0..=180.0)",
                    longitude
                )
            }
            Error::AltitudeOutOfRange { altitude } => {
                write!(
                    f,
                    "Altitude '{}' is out of range (valid: -33,554,432.0..=33,554,432.0)",
                    altitude
                )
            }

            // --- 新規追加: 幾何形状エラーメッセージ ---
            Error::TooFewPoints(n) => {
                write!(
                    f,
                    "Polygon must have at least 4 points (start==end), but got {}",
                    n
                )
            }
            Error::NotClosedRing => {
                write!(f, "Polygon ring is not closed (first point != last point)")
            }
            Error::CollinearPoints => {
                write!(
                    f,
                    "Polygon points are collinear (cannot compute normal vector)"
                )
            }
            Error::NotPlanar => {
                write!(f, "Polygon vertices are not on the same plane")
            }
            Error::EmptySolid => {
                write!(f, "Solid must have at least one surface")
            }
            Error::DegenerateEdge(idx) => {
                write!(
                    f,
                    "Surface index {} contains a degenerate edge (length is 0)",
                    idx
                )
            }
            Error::OpenHoleDetected => {
                write!(
                    f,
                    "Solid is not watertight (hole detected: edge used only once)"
                )
            }
            Error::NonManifoldEdge => {
                write!(
                    f,
                    "Solid contains non-manifold edges or inconsistent normals (edge used >2 times or same direction twice)"
                )
            }
        }
    }
}

impl error::Error for Error {}
