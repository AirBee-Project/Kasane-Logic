use thiserror::Error;

/// Kasane-logicで発生するエラー型
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("ZoomLevel '{z}' is out of range (valid: 0..=60)")]
    ZOutOfRange { z: u8 },

    #[error("F coordinate '{f}' is out of range for ZoomLevel '{z}'")]
    FOutOfRange { z: u8, f: i64 },

    #[error("X coordinate '{x}' is out of range for ZoomLevel '{z}'")]
    XOutOfRange { z: u8, x: u64 },

    #[error("Y coordinate '{y}' is out of range for ZoomLevel '{z}'")]
    YOutOfRange { z: u8, y: u64 },

    #[error("Latitude '{latitude}' is out of range (valid: -85.0511..=85.0511)")]
    LatitudeOutOfRange { latitude: f64 },

    #[error("Longitude '{longitude}' is out of range (valid: -180.0..=180.0)")]
    LongitudeOutOfRange { longitude: f64 },

    #[error("Altitude '{altitude}' is out of range (valid: -33,554,432.0..=33,554,432.0)")]
    AltitudeOutOfRange { altitude: f64 },
}
