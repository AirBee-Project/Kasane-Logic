use std::error;
use std::fmt;

/// 本クレートで発生し得るエラーを表す。
#[derive(Debug, PartialEq)] // PartialEqはテスト等で便利ですが、f64を含む場合は注意が必要です
pub enum Error {
    // --- 既存の座標・ズームレベルエラー ---
    /// ズームレベルが有効範囲（0..=31）外であることを示す。
    ZOutOfRange { z: u8 },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して有効範囲外であることを示す。
    FOutOfRange { z: u8, f: i64 },

    /// X 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    XOutOfRange { z: u8, x: u64 },

    /// Y 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    YOutOfRange { z: u8, y: u64 },

    /// Tがu64を超えてオーバーフローしたことを示す。
    TOutOfRange { current: u64, offset: i64 },

    /// 緯度が有効範囲外であることを表す。
    LatitudeOutOfRange { latitude: f64 },

    /// 経度が有効範囲外であることを表す。
    LongitudeOutOfRange { longitude: f64 },

    /// 高度が有効範囲外であることを表す。
    AltitudeOutOfRange { altitude: f64 },

    ///Solidの作成時に起きるエラー
    SolidNotWatertight {
        /// 問題のあるエッジの数（奇数回しか出現しなかったエッジの数）
        open_edge_count: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Error::SolidNotWatertight { open_edge_count } => {
                write!(
                    f,
                    "Solid is not watertight (closed). Found {} open edges.",
                    open_edge_count
                )
            }
            Error::TOutOfRange { current, offset } => {
                let direction = if *offset >= 0 { "add" } else { "subtract" };
                write!(
                    f,
                    "Time calculation error: failed to {} offset '{}' to/from current time '{}' (resulting in overflow/underflow)",
                    direction,
                    offset.abs(),
                    current
                )
            }
        }
    }
}

impl error::Error for Error {}
