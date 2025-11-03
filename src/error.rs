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
}
