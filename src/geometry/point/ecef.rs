use std::{fmt, ops::Sub};

use crate::{
    SingleId,
    error::Error,
    geometry::{
        constants::{WGS84_A, WGS84_E2, WGS84_F},
        point::{Point, coordinate::Coordinate},
    },
};

/// 地心直交座標系（ECEF: Earth-spatial_centered, Earth-Fixed）における座標を表す。
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
    /// assert_eq!(ecef.x(), 10.0);
    /// assert_eq!(ecef.y(), 20.0);
    /// assert_eq!(ecef.z(), 30.0);
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
    /// assert_eq!(ecef.x(), 1.0);
    /// ```
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Y 成分を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(0.0, 2.0, 0.0);
    /// assert_eq!(ecef.y(), 2.0);
    /// ```
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Z 成分を返す。
    ///
    /// # Examples
    /// ```
    /// # use kasane_logic::Ecef;
    ///
    /// let ecef = Ecef::new(0.0, 0.0, 3.0);
    /// assert_eq!(ecef.z(), 3.0);
    /// ```
    pub fn z(&self) -> f64 {
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
    /// assert_eq!(ecef.x(), 5.0);
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
    /// assert_eq!(ecef.y(), 6.0);
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
    /// assert_eq!(ecef.z(), 7.0);
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
        ((self.x() - other.x()).powi(2)
            + (self.y() - other.y()).powi(2)
            + (self.z() - other.z()).powi(2))
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

    /// Ecefが同じ位置にあるかを判定します
    /// 2点間の直線距離が epsilon 以内にあるかを判定します
    pub fn eq_epsilon(&self, other: &Ecef, epsilon: f64) -> bool {
        let distance_squared = self.distance(other);
        distance_squared < epsilon * epsilon
    }

    /// 内積（ドット積）を計算する。
    pub fn dot(&self, other: &Ecef) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 指定された軸インデックス (0=x, 1=y, 2=z) の成分を返す。
    /// 投影計算などで軸を動的に扱いたい場合に有用。
    pub fn get_component(&self, index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    /// 指定された2つの軸（u, v）に基づいて 2D 平面に投影した座標を返す。
    pub fn project_2d(&self, u_axis: usize, v_axis: usize) -> (f64, f64) {
        (self.get_component(u_axis), self.get_component(v_axis))
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
