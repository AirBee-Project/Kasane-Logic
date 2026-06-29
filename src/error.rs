use alloc::string::String;

use core::{error, fmt};

/// 本クレートで発生し得るエラーを表す最上位の型。
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// 空間IDまわりのエラー。
    SpatialId(SpatialIdError),

    /// Geometry まわりのエラー。
    Geometry(GeometryError),
}

/// Geometry 関連で発生するエラー。
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryError {
    /// 緯度が有効範囲外であることを表す。
    LatitudeOutOfRange { latitude: f64 },

    /// 経度が有効範囲外であることを表す。
    LongitudeOutOfRange { longitude: f64 },

    /// 高度が有効範囲外であることを表す。
    AltitudeOutOfRange { altitude: f64 },

    /// Solid が watertight ではないことを表す。
    SolidNotWatertight {
        /// 問題のあるエッジの数（奇数回しか出現しなかったエッジの数）
        open_edge_count: usize,
    },

    ///半径が負であることを表す
    RadiusNegative { radius: f64 },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して有効範囲外であることを示す。
    FractionalFOutOfRange { z: u8, f: f64 },

    /// X 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    FractionalXOutOfRange { z: u8, x: f64 },

    /// Y 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    FractionalYOutOfRange { z: u8, y: f64 },
}

/// SpatialId 関連で発生するエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialIdError {
    /// ズームレベルが有効範囲（0..=30）外であることを示す。
    ZOutOfRange { z: u8 },

    /// ある操作に対して、現在のズームレベルと要求されたズームレベルの上下関係が不正であることを示す。
    ///
    /// たとえば、より深いズームレベルを要求する操作に対して浅いズームレベルを指定した場合などに使う。
    ZoomLevelTransitionOutOfRange { current_z: u8, target_z: u8 },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して有効範囲外であることを示す。
    FOutOfRange { z: u8, f: i32 },

    /// X 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    XOutOfRange { z: u8, x: u32 },

    /// Y 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    YOutOfRange { z: u8, y: u32 },

    /// 時間方向が0-u64::MAX的有効範囲外であることを示す。
    /// 0=<i×t=<u64::MAXを満たす必要がある
    TOutOfRange { i: u64, t: u64 },

    /// 時間間隔 `i` に 0 を指定した場合のエラー。
    TIntervalError { i: u64 },
    /// 文字列表現を空間 ID として解釈できないことを示す。
    ParseSpatialIdFormat { kind: &'static str, input: String },

    /// シャードのマージに渡された2つが、指定親領域の正当な兄弟（下半分/上半分）でないことを示す。
    InvalidShardMerge,
}

impl From<GeometryError> for Error {
    fn from(value: GeometryError) -> Self {
        Self::Geometry(value)
    }
}

impl From<SpatialIdError> for Error {
    fn from(value: SpatialIdError) -> Self {
        Self::SpatialId(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SpatialId(inner) => inner.fmt(f),
            Error::Geometry(inner) => inner.fmt(f),
        }
    }
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::LatitudeOutOfRange { latitude } => {
                write!(
                    f,
                    "Latitude '{}' is out of range (valid: -85.0511..=85.0511)",
                    latitude
                )
            }
            GeometryError::LongitudeOutOfRange { longitude } => {
                write!(
                    f,
                    "Longitude '{}' is out of range (valid: -180.0..=180.0)",
                    longitude
                )
            }
            GeometryError::AltitudeOutOfRange { altitude } => {
                write!(
                    f,
                    "Altitude '{}' is out of range (valid: -33,554,432.0..=33,554,432.0)",
                    altitude
                )
            }
            GeometryError::SolidNotWatertight { open_edge_count } => {
                write!(
                    f,
                    "Solid is not watertight (closed). Found {} open edges.",
                    open_edge_count
                )
            }
            GeometryError::RadiusNegative { radius } => {
                write!(f, "Radius need to be positive (radius = {}).", radius)
            }
            GeometryError::FractionalFOutOfRange { z, f: fv } => {
                write!(
                    f,
                    "Fractional F coordinate '{}' is out of range for ZoomLevel '{}'",
                    fv, z
                )
            }
            GeometryError::FractionalXOutOfRange { z, x } => {
                write!(
                    f,
                    "Fractional X coordinate '{}' is out of range for ZoomLevel '{}'",
                    x, z
                )
            }
            GeometryError::FractionalYOutOfRange { z, y } => {
                write!(
                    f,
                    "Fractional Y coordinate '{}' is out of range for ZoomLevel '{}'",
                    y, z
                )
            }
        }
    }
}

impl fmt::Display for SpatialIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpatialIdError::ZOutOfRange { z } => {
                write!(f, "ZoomLevel '{}' is out of range (valid: 0..=30)", z)
            }
            SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z,
                target_z,
            } => {
                write!(
                    f,
                    "Target zoom level '{}' is invalid for current zoom level '{}'",
                    target_z, current_z
                )
            }
            SpatialIdError::FOutOfRange { z, f: fv } => {
                write!(
                    f,
                    "F coordinate '{}' is out of range for ZoomLevel '{}'",
                    fv, z
                )
            }
            SpatialIdError::XOutOfRange { z, x } => {
                write!(
                    f,
                    "X coordinate '{}' is out of range for ZoomLevel '{}'",
                    x, z
                )
            }
            SpatialIdError::YOutOfRange { z, y } => {
                write!(
                    f,
                    "Y coordinate '{}' is out of range for ZoomLevel '{}'",
                    y, z
                )
            }
            SpatialIdError::TOutOfRange { i, t } => {
                write!(f, "i × t overflows u64 (i={}, t={}).", i, t)
            }
            SpatialIdError::TIntervalError { i } => {
                write!(
                    f,
                    "Time interval i is must u64::MAX or 86400 or 3600 or 60 or 1 (i={}).",
                    i
                )
            }
            SpatialIdError::ParseSpatialIdFormat { kind, input } => {
                write!(f, "{} '{}' has invalid display format", kind, input)
            }
            SpatialIdError::InvalidShardMerge => {
                write!(
                    f,
                    "the two shards are not valid siblings of the parent region"
                )
            }
        }
    }
}

impl error::Error for Error {}

impl error::Error for GeometryError {}

impl error::Error for SpatialIdError {}
