use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Error, Serialize, Deserialize)]

pub enum Error {
    #[error("ZoomLevel '{zoom_level}' is out of range (valid: 0..=60)")]
    ZoomLevelOutOfRange { zoom_level: u8 },

    #[error("F coordinate '{f}' is out of range for ZoomLevel '{z}'")]
    FOutOfRange { f: i64, z: u8 },

    #[error("X coordinate '{x}' is out of range for ZoomLevel '{z}'")]
    XOutOfRange { x: u64, z: u8 },

    #[error("Y coordinate '{y}' is out of range for ZoomLevel '{z}'")]
    YOutOfRange { y: u64, z: u8 },

    #[error("Latitude '{latitude}' is out of range (valid: -90.0..=90.0)")]
    LatitudeOutOfRange { latitude: f64 },

    #[error("Longitude '{longitude}' is out of range (valid: -180.0..=180.0)")]
    LongitudeOutOfRange { longitude: f64 },

    #[error("Altitude '{altitude}' is out of range (valid: -33,554,432.0..=33,554,432.0)")]
    AltitudeOutOfRange { altitude: f64 },
}
