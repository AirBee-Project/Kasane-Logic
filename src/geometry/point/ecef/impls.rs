use std::{fmt, ops::Sub};

use crate::{Coordinate, Ecef, Error, Point, WGS84_A, WGS84_E2, WGS84_F};

impl fmt::Debug for Ecef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ecef")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
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
