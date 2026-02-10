use std::{fmt, ops::Sub};

use crate::{
    SingleId,
    error::Error,
    geometry::{
        constants::{WGS84_A, WGS84_E2, WGS84_F},
        point::{Point, coordinate::Coordinate},
    },
};

/// 地心直交座標系（ECEF: Earth-Centered, Earth-Fixed）における座標を表す。
///
/// 原点は地球の重心にあり、
/// * X 軸は赤道面上で本初子午線方向
/// * Y 軸は赤道面上で東経 90 度方向
/// * Z 軸は北極方向
///
/// 単位はすべてメートル。
#[derive(Clone, Copy)]
pub struct Ecef {
    x: f64,
    y: f64,
    z: f64,
}

impl fmt::Debug for Ecef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ecef")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}

impl Ecef {
    /// 指定された XYZ 成分から [`Ecef`] を生成する。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(10.0, 20.0, 30.0);
    ///
    /// assert_eq!(ecef.as_x(), 10.0);
    /// assert_eq!(ecef.as_y(), 20.0);
    /// assert_eq!(ecef.as_z(), 30.0);
    /// ```
    pub fn new(x: f64, y: f64, z: f64) -> Ecef {
        Ecef { x, y, z }
    }
    /// X 成分を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(1.0, 0.0, 0.0);
    /// assert_eq!(ecef.as_x(), 1.0);
    /// ```
    pub fn as_x(&self) -> f64 {
        self.x
    }

    /// Y 成分を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(0.0, 2.0, 0.0);
    /// assert_eq!(ecef.as_y(), 2.0);
    /// ```
    pub fn as_y(&self) -> f64 {
        self.y
    }

    /// Z 成分を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(0.0, 0.0, 3.0);
    /// assert_eq!(ecef.as_z(), 3.0);
    /// ```
    pub fn as_z(&self) -> f64 {
        self.z
    }

    /// X 成分を設定する。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let mut ecef = Ecef::new(0.0, 0.0, 0.0);
    /// ecef.set_x(5.0);
    ///
    /// assert_eq!(ecef.as_x(), 5.0);
    /// ```
    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }

    /// Y 成分を設定する。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let mut ecef = Ecef::new(0.0, 0.0, 0.0);
    /// ecef.set_y(6.0);
    ///
    /// assert_eq!(ecef.as_y(), 6.0);
    /// ```
    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }

    /// Z 成分を設定する。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let mut ecef = Ecef::new(0.0, 0.0, 0.0);
    /// ecef.set_z(7.0);
    ///
    /// assert_eq!(ecef.as_z(), 7.0);
    /// ```
    pub fn set_z(&mut self, z: f64) {
        self.z = z;
    }

    /// この ECEF 座標を、指定されたズームレベルの [`SingleId`] に変換する。
    pub fn to_single_id(&self, z: u8) -> Result<SingleId, Error> {
        let coordinate: Coordinate = (*self).try_into()?;
        Ok(coordinate.to_single_id(z)?)
    }

    /// 他の [`Ecef`] 座標との距離をメートル単位で返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let a = Ecef::new(0.0, 0.0, 0.0);
    /// let b = Ecef::new(3.0, 4.0, 0.0);
    ///
    /// assert_eq!(a.distance(&b), 5.0);
    /// ```
    pub fn distance(&self, other: &Ecef) -> f64 {
        ((self.as_x() - other.as_x()).powi(2)
            + (self.as_y() - other.as_y()).powi(2)
            + (self.as_z() - other.as_z()).powi(2))
        .sqrt()
    }

    ///他の[Ecef]型との外積を取る。
    pub fn cross(&self, other: &Ecef) -> Ecef {
        Ecef {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    // 原点からの距離の2乗を取得する。
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// 3点以上の地点が一直線上にないことを判定する
    /// epsilon: 直線からの許容するズレの距離（例: 0.1 なら 10cm）
    pub fn is_not_collinear(points: &[Ecef], epsilon: f64) -> bool {
        if points.len() < 3 {
            return false;
        }

        let p0 = points[0];
        let epsilon_sq = epsilon * epsilon;

        let base_vec = points
            .iter()
            .skip(1)
            .map(|&p| p - p0)
            .find(|v| v.norm_squared() > epsilon_sq);

        let ab = match base_vec {
            Some(v) => v,
            None => return false, // すべての点が p0 から epsilon 以内にある
        };

        let ab_norm_sq = ab.norm_squared();

        // 他のすべての点について、直線(p0, ab)からの距離を計算
        for &p in points.iter().skip(1) {
            let ac = p - p0;
            let cross_prod = ab.cross(&ac);
            let dist_to_line_sq = cross_prod.norm_squared() / ab_norm_sq;
            if dist_to_line_sq > epsilon_sq {
                return true;
            }
        }

        false
    }

    /// Ecefが同じ位置にあるかを判定します
    /// 2点間の直線距離が epsilon 以内にあるかを判定します
    pub fn eq_epsilon(&self, other: &Ecef, epsilon: f64) -> bool {
        let distance_squared = self.distance(other);
        distance_squared < epsilon * epsilon
    }
}

impl TryFrom<Ecef> for Coordinate {
    type Error = Error;
    /// 地心直交座標系（ECEF）から地理座標（緯度・経度・高度）への変換。
    fn try_from(value: Ecef) -> Result<Self, Self::Error> {
        let x = value.x;
        let y = value.y;
        let z = value.z;

        let lon = y.atan2(x);
        let p = (x * x + y * y).sqrt();

        // 緯度の初期値（Bowring）
        let mut lat = (z / p).atan2(1.0 - WGS84_F);
        let mut h = 0.0;

        for _ in 0..10 {
            let sin_lat = lat.sin();
            let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();
            h = p / lat.cos() - n;

            let new_lat = (z + WGS84_E2 * n * sin_lat).atan2(p);

            if (new_lat - lat).abs() < 1e-12 {
                lat = new_lat;
                break;
            }
            lat = new_lat;
        }

        Coordinate::new(lat.to_degrees(), lon.to_degrees(), h)
    }
}

impl Point for Ecef {}

// Ecef - Ecef の実装
impl Sub for Ecef {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// 参照 (&Ecef - &Ecef) も実装しておくと便利です
impl<'a, 'b> Sub<&'b Ecef> for &'a Ecef {
    type Output = Ecef;

    fn sub(self, other: &'b Ecef) -> Self::Output {
        Ecef {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
