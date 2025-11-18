use crate::{error::Error, space_time_id::SpaceTimeId};

/// 地球上の位置を表す列挙型
///
/// 座標系（緯度経度高度）またはECEF座標系で表現できる。
pub enum Point {
    /// 緯度経度高度の座標系
    Coordinate(Coordinate),
    /// ECEF（Earth-Centered, Earth-Fixed）座標系
    ECEF(ECEF),
}

impl Point {
    /// 座標系に変換する
    ///
    /// # 戻り値
    /// Coordinate形式の座標
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

/// 緯度経度高度で表される座標
///
/// WGS-84測地系を使用した地理座標系。
#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

/// ECEF（Earth-Centered, Earth-Fixed）座標系で表される座標
///
/// 地球中心を原点とした直交座標系。
#[derive(Debug, Clone, Copy)]
pub struct ECEF {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl ECEF {
    /// ECEF座標を生成する
    ///
    /// # 引数
    /// * `x` - X座標（メートル）
    /// * `y` - Y座標（メートル）
    /// * `z` - Z座標（メートル）
    pub fn new(x: f64, y: f64, z: f64) -> ECEF {
        ECEF { x, y, z }
    }

    /// ECEF座標を緯度経度高度に変換する
    ///
    /// WGS-84測地系を使用してNewton-Raphson法で反復計算を行う。
    ///
    /// # 戻り値
    /// 変換後のCoordinate
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

    /// ECEF座標から時空間IDを生成する
    ///
    /// # 引数
    /// * `z` - ズームレベル
    pub fn to_id(&self, z: u8) -> SpaceTimeId {
        self.to_coordinate().to_id(z)
    }
}

impl Coordinate {
    /// 緯度経度高度から座標を生成する
    ///
    /// 各値が有効範囲内かを検証してから生成する。
    ///
    /// # 引数
    /// * `latitude` - 緯度（度）-90.0..=90.0
    /// * `longitude` - 経度（度）-180.0..=180.0
    /// * `altitude` - 高度（メートル）-33,554,432.0..=33,554,432.0
    ///
    /// # エラー
    /// 値が範囲外の場合にエラーを返す
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

    /// 緯度経度高度をECEF座標に変換する
    ///
    /// WGS-84測地系を使用して変換を行う。
    ///
    /// # 戻り値
    /// 変換後のECEF座標
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

    /// 座標から時空間IDを生成する
    ///
    /// 緯度経度高度を指定されたズームレベルで時空間IDに変換する。
    ///
    /// # 引数
    /// * `z` - ズームレベル
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
