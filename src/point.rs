use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{error::Error, space_time_id::SpaceTimeId};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS)]
pub enum Point {
    Coordinate(Coordinate),
    ECEF(ECEF),
}

impl Point {
    /// Coordinate形式に変換
    pub fn to_coordinate(&self) -> Coordinate {
        match self {
            Point::Coordinate(coordinate) => *coordinate,
            Point::ECEF(ecef) => ecef.to_coordinate(),
        }
    }

    pub fn to_ecef(&self) -> ECEF {
        match self {
            Point::Coordinate(coordinate) => coordinate.to_ecef(),
            Point::ECEF(ecef) => *ecef,
        }
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
pub struct ECEF {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ECEF {
    /// 新しいECEF座標を作成
    pub fn new(x: f64, y: f64, z: f64) -> ECEF {
        ECEF { x, y, z }
    }

    pub fn to_coordinate(&self) -> Coordinate {
        let a = 6378137.0_f64; // 長半径
        let inv_f = 298.257223563_f64;
        let f = 1.0 / inv_f;
        let b = a * (1.0 - f);
        let e2 = 1.0 - (b * b) / (a * a);

        let x = self.x;
        let y = self.y;
        let z = self.z;

        let lon = y.atan2(x);
        let p = (x * x + y * y).sqrt();

        // 緯度の初期値（Bowring の公式）
        let mut lat = (z / p).atan2(1.0 - f);
        let mut h = 0.0;

        // Newton-Raphson 反復
        for _ in 0..10 {
            let sin_lat = lat.sin();
            let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
            h = p / lat.cos() - n;
            let new_lat = (z + e2 * n * sin_lat).atan2(p);

            // 収束チェック（1e-12 ≈ 数 mm）
            if (new_lat - lat).abs() < 1e-12 {
                lat = new_lat;
                break;
            }
            lat = new_lat;
        }

        Coordinate {
            latitude: lat.to_degrees(),
            longitude: lon.to_degrees(),
            altitude: h,
        }
    }

    /// 指定されたズームレベルで時空間IDに変換（ECEF版）
    pub fn to_id(&self, z: u8) -> SpaceTimeId {
        self.to_coordinate().to_id(z)
    }
}

impl Coordinate {
    /// 新しい座標を作成（範囲チェック付き）
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> Result<Self, Error> {
        if !(-90.0..=90.0).contains(&latitude) {
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

    pub fn to_ecef(&self) -> ECEF {
        // WGS-84 定数
        let a: f64 = 6_378_137.0;
        let inv_f: f64 = 298.257_223_563;
        let f = 1.0 / inv_f;
        let b = a * (1.0 - f);
        let e2 = 1.0 - (b * b) / (a * a);

        let lat = self.latitude.to_radians();
        let lon = self.longitude.to_radians();
        let h = self.altitude;

        let sin_lat = lat.sin();
        let cos_lat = lat.cos();
        let cos_lon = lon.cos();
        let sin_lon = lon.sin();

        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();

        let x_f64 = (n + h) * cos_lat * cos_lon;
        let y_f64 = (n + h) * cos_lat * sin_lon;
        let z_f64 = (n * (1.0 - e2) + h) * sin_lat;

        ECEF {
            x: x_f64,
            y: y_f64,
            z: z_f64,
        }
    }

    /// 指定されたズームレベルで時空間IDに変換（Coordinate版）
    pub fn to_id(&self, z: u8) -> SpaceTimeId {
        let lat = self.latitude;
        let lon = self.longitude;
        let alt = self.altitude;

        // ---- 高度 h -> f (Python の h_to_f を Rust に移植) ----
        let factor = 2_f64.powi(z as i32 - 25); // 2^(z-25)
        let f_id = (factor * alt).floor() as i64;

        // ---- 経度 lon -> x ----
        let n = 2u64.pow(z as u32) as f64;
        let x_id = ((lon + 180.0) / 360.0 * n).floor() as u64;

        // ---- 緯度 lat -> y (Web Mercator) ----
        let lat_rad = lat.to_radians();
        let y_id = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0
            * n)
            .floor() as u64;

        SpaceTimeId {
            z,
            f: [f_id, f_id],
            x: [x_id, x_id],
            y: [y_id, y_id],
            i: 0,
            t: [0, u64::MAX],
        }
    }
}
