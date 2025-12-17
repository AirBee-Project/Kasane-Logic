use std::fmt;

use crate::{
    error::Error,
    geometry::{
        constants::{WGS84_A, WGS84_E2, WGS84_INV_F},
        ecef::Ecef,
    },
    id::space_id::single::SingleID,
};

/// `Coordinate` 型は、緯度・経度・高度によって点の位置を表現するための型です。
/// 内部的には下記のような構造体として定義されており、各フィールドを非公開とすることで、
/// 空間 ID 上で扱える座標に対する制約が常に満たされるようにしています。
///
/// この型は `PartialOrd` を実装していますが、これは主に `BTreeSet` や `BTreeMap`
/// といった順序付きコレクションにおける格納および探索を目的としたものです。
/// 空間的な位置関係における「大小」を意味するものではありません。
/// ```
/// pub struct Coordinate {
///     latitude: f64,
///     longitude: f64,
///     altitude: f64,
/// }
/// ```
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Coordinate {
    latitude: f64,
    longitude: f64,
    altitude: f64,
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Coordinate")
            .field("latitude", &self.latitude)
            .field("longitude", &self.longitude)
            .field("altitude", &self.altitude)
            .finish()
    }
}

impl Coordinate {
    /// 指定された緯度・経度・高度から `Coordinate` を生成します。
    ///
    /// 各引数は、空間 ID 上で扱える座標として有効な範囲に収まっている必要があります。
    /// 範囲外の値が指定された場合、この関数は対応するエラーを返します。
    ///
    /// # 引数
    /// * `latitude` - 緯度（-85.0511 〜 85.0511）
    /// * `longitude` - 経度（-180.0 〜 180.0）
    /// * `altitude` - 高度（-33,554,432.0 〜 33,554,432.0）
    ///
    /// # 戻り値
    /// * 有効な値が指定された場合は `Ok(Coordinate)` を返します
    /// * いずれかの値が範囲外の場合は、対応する `Error` を返します
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

    /// 値の妥当性検証を行わずに `Coordinate` を生成します。
    ///
    /// この関数は緯度・経度・高度に対する範囲チェックを一切行いません。
    /// 呼び出し側は、渡す値が空間 ID 上で扱える有効な範囲に収まっていることを
    /// 保証する責任を負います。
    ///
    /// # Safety
    /// この関数は `unsafe` です。
    /// 不正な値を指定した場合、`Coordinate` が前提としている不変条件が破られ、
    /// 以降の処理で未定義な振る舞いまたは論理的な不整合を引き起こす可能性があります。
    /// そのため、入力値の正当性が外部で十分に検証されている場合にのみ使用してください。
    pub unsafe fn uncheck_new(latitude: f64, longitude: f64, altitude: f64) -> Coordinate {
        Coordinate {
            latitude,
            longitude,
            altitude,
        }
    }

    /// 緯度を返します。
    pub fn as_latitude(&self) -> f64 {
        self.latitude
    }

    ///経度を返します。
    pub fn as_longitude(&self) -> f64 {
        self.longitude
    }

    ///高度を返します。
    pub fn as_altitude(&self) -> f64 {
        self.altitude
    }

    /// 緯度を設定します。
    pub fn set_latitude(&mut self, latitude: f64) -> Result<(), Error> {
        if !(-85.0511..=85.0511).contains(&latitude) {
            return Err(Error::LatitudeOutOfRange { latitude });
        }
        self.latitude = latitude;
        Ok(())
    }

    /// 経度を設定します。
    pub fn set_longitude(&mut self, longitude: f64) -> Result<(), Error> {
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(Error::LongitudeOutOfRange { longitude });
        }
        self.longitude = longitude;
        Ok(())
    }

    /// 高度を設定します。
    pub fn set_altitude(&mut self, altitude: f64) -> Result<(), Error> {
        if !(-33_554_432.0..=33_554_432.0).contains(&altitude) {
            return Err(Error::AltitudeOutOfRange { altitude });
        }
        self.altitude = altitude;
        Ok(())
    }

    /// この座標を、指定されたズームレベルに対応する `SingleID` に変換します。
    ///
    /// 緯度・経度・高度をそれぞれ空間 ID の各成分（`x`, `y`, `f`）へ変換し、
    /// ズームレベル `z` を含む `SingleID` を生成します。
    ///
    /// # 引数
    /// * `z` - 空間 ID のズームレベル
    ///
    /// # 戻り値
    /// * 指定されたズームレベルに対応する `SingleID`
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
    /// [`Coordinate`]を[`Ecef`]へ変換します。
    fn from(value: Coordinate) -> Self {
        let lat = value.latitude.to_radians();
        let lon = value.longitude.to_radians();
        let h = value.altitude;

        let sin_lat = lat.sin();
        let cos_lat = lat.cos();
        let sin_lon = lon.sin();
        let cos_lon = lon.cos();

        let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();

        let x = (n + h) * cos_lat * cos_lon;
        let y = (n + h) * cos_lat * sin_lon;
        let z = (n * (1.0 - WGS84_E2) + h) * sin_lat;

        Ecef { x, y, z }
    }
}

/// `Coordinate` の既定値を返します。
///
/// 緯度・経度・高度のすべてが `0.0` に設定された座標
///（赤道・本初子午線上、高度 0）を表します。
/// この値は常に有効な範囲内に収まります。
impl Default for Coordinate {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        }
    }
}
