use std::error;
use std::fmt;

/// 本クレートで発生し得るエラーを表す。
#[derive(Debug, PartialEq)]
pub enum Error {
    /// ズームレベルが有効範囲（0..=31）外であることを示す。
    ZOutOfRange {
        z: u8,
    },

    /// 高度方向インデックス `f` が、指定されたズームレベルに対して有効範囲外であることを示す。
    FOutOfRange {
        z: u8,
        f: i32,
    },

    /// X 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    XOutOfRange {
        z: u8,
        x: u32,
    },

    /// Y 方向インデックスが、指定されたズームレベルに対して有効範囲外であることを示す。
    YOutOfRange {
        z: u8,
        y: u32,
    },

    /// 緯度が有効範囲外であることを表す。
    LatitudeOutOfRange {
        latitude: f64,
    },

    /// 経度が有効範囲外であることを表す。
    LongitudeOutOfRange {
        longitude: f64,
    },

    /// 高度が有効範囲外であることを表す。
    AltitudeOutOfRange {
        altitude: f64,
    },

    TooFewPoints(usize),
    NotClosedRing,
    NotPlanar,
    CollinearPoints,
    AllPointsIdentical,
    AllPointsCollinear,
    EmptySolid,
    NoSurfaces,
    NotClosedSolid,
    NonManifoldSolid(usize),
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
            Error::TooFewPoints(n) => write!(f, "Too few points: {} (need at least 3)", n),
            Error::NotClosedRing => write!(f, "Ring is not closed"),
            Error::NotPlanar => write!(f, "Points do not form a planar surface"),
            Error::CollinearPoints => write!(f, "Points are collinear"),
            Error::AllPointsIdentical => write!(f, "All points are identical"),
            Error::AllPointsCollinear => write!(f, "All points are collinear"),
            Error::EmptySolid => write!(f, "EmptySolid"),
            Error::NoSurfaces => write!(f, "NoSurfaces"),
            Error::NotClosedSolid => write!(f, "NotClosedSolid"),
            Error::NonManifoldSolid(_) => write!(f, "NonManifoldSolid"),
        }
    }
}

impl error::Error for Error {}
