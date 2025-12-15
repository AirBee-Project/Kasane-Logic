use crate::{
    error::Error,
    geometry::{
        constants::{WGS84_A, WGS84_INV_F},
        ecef::Ecef,
    },
    id::space_id::single::SingleID,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinate {
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) altitude: f64,
}

impl Coordinate {
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> Result<Self, Error> {
        if !(-85.0511..=85.0511).contains(&latitude) {
            return Err(Error::LatitudeOutOfRange { latitude });
        }

        if !(-180.0..=180.0).contains(&longitude) {
            return Err(Error::LongitudeOutOfRange { longitude });
        }

        if !(-33_554_432.0..=33_554_432.0).contains(&altitude) {
            return Err(Error::AltitudeOutOfRange { altitude });
        }

        Ok(Self {
            latitude,
            longitude,
            altitude,
        })
    }

    pub unsafe fn uncheck_new(latitude: f64, longitude: f64, altitude: f64) -> Coordinate {
        Coordinate {
            latitude,
            longitude,
            altitude,
        }
    }

    pub fn as_latitude(&self) -> f64 {
        self.latitude
    }

    pub fn as_longitude(&self) -> f64 {
        self.longitude
    }

    pub fn as_altitude(&self) -> f64 {
        self.altitude
    }

    pub fn to_id(&self, z: u8) -> SingleID {
        let lat = self.latitude;
        let lon = self.longitude;
        let alt = self.altitude;

        // ---- 高度 h -> f (Python の h_to_f を Rust に移植) ----
        let factor = 2_f64.powi(z as i32 - 25); // 2^(z-25)
        let f = (factor * alt).floor() as i64;

        // ---- 経度 lon -> x ----
        let n = 2u64.pow(z as u32) as f64;
        let x = ((lon + 180.0) / 360.0 * n).floor() as u64;

        // ---- 緯度 lat -> y (Web Mercator) ----
        let lat_rad = lat.to_radians();
        let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0
            * n)
            .floor() as u64;

        SingleID { z, f, x, y }
    }
}

impl From<Coordinate> for Ecef {
    fn from(value: Coordinate) -> Self {
        let f = 1.0 / WGS84_INV_F;
        let b = WGS84_A * (1.0 - f);
        let e2 = 1.0 - (b * b) / (WGS84_A * WGS84_A);

        let lat = value.latitude.to_radians();
        let lon = value.longitude.to_radians();
        let h = value.altitude;

        let sin_lat = lat.sin();
        let cos_lat = lat.cos();
        let cos_lon = lon.cos();
        let sin_lon = lon.sin();

        let n = WGS84_A / (1.0 - e2 * sin_lat * sin_lat).sqrt();

        let x_f64 = (n + h) * cos_lat * cos_lon;
        let y_f64 = (n + h) * cos_lat * sin_lon;
        let z_f64 = (n * (1.0 - e2) + h) * sin_lat;

        Ecef {
            x: x_f64,
            y: y_f64,
            z: z_f64,
        }
    }
}
