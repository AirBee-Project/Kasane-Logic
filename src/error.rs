use thiserror::Error;

/// 本クレートで発生し得るエラーを表します。
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// ズームレベルが有効範囲（0..=60）外であることを示します。
    #[error("ZoomLevel '{z}' is out of range (valid: 0..=60)")]
    ZOutOfRange { z: u8 },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して
    /// 有効範囲外であることを示します。
    #[error("F coordinate '{f}' is out of range for ZoomLevel '{z}'")]
    FOutOfRange { z: u8, f: i64 },

    /// X 方向インデックスが、指定されたズームレベルに対して
    /// 有効範囲外であることを示します。
    #[error("X coordinate '{x}' is out of range for ZoomLevel '{z}'")]
    XOutOfRange { z: u8, x: u64 },

    /// Y 方向インデックスが、指定されたズームレベルに対して
    /// 有効範囲外であることを示します。
    #[error("Y coordinate '{y}' is out of range for ZoomLevel '{z}'")]
    YOutOfRange { z: u8, y: u64 },

    /// 経度が有効範囲外であることを示します。
    ///
    /// 有効範囲は `-180.0 ..= 180.0` です。
    #[error("Latitude '{latitude}' is out of range (valid: -85.0511..=85.0511)")]
    LatitudeOutOfRange { latitude: f64 },

    /// 高度が有効範囲外であることを示します。
    ///
    /// 有効範囲は空間 ID の設計上、
    /// `-33,554,432.0 ..= 33,554,432.0` に制限されています。
    #[error("Longitude '{longitude}' is out of range (valid: -180.0..=180.0)")]
    LongitudeOutOfRange { longitude: f64 },

    /// 緯度が有効範囲外であることを示します。
    ///
    /// 有効範囲は Web Mercator 投影を前提とした`-85.0511 ..= 85.0511` です。
    #[error("Altitude '{altitude}' is out of range (valid: -33,554,432.0..=33,554,432.0)")]
    AltitudeOutOfRange { altitude: f64 },
}
