use crate::{Coordinate, Ecef, Point, RangeId, WGS84_A, WGS84_E2, geometry::traits::Geometry};
use std::fmt;

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Coordinate")
            .field("latitude", &self.latitude)
            .field("longitude", &self.longitude)
            .field("altitude", &self.altitude)
            .finish()
    }
}

impl From<Coordinate> for Ecef {
    /// [`Coordinate`]を[`Ecef`]への変換。
    /// ```
    /// # use kasane_logic::{Coordinate,Ecef};
    /// let coord = Coordinate::new(43.068564, 41.3507138, 30.0).unwrap();
    /// let ecef: Ecef = coord.into();
    /// print!("{},{},{}", ecef.x(), ecef.y(), ecef.z());
    /// assert_eq!(ecef.x(), 3503254.6369501497);
    /// assert_eq!(ecef.y(), 3083182.6924748584);
    /// assert_eq!(ecef.z(), 4333089.862951963);
    /// ```

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

        Ecef::new(x, y, z)
    }
}

/// 緯度・経度・高度のすべてが `0.0` に設定された座標を返す。
impl Default for Coordinate {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        }
    }
}

impl Geometry for Coordinate {
    fn center(&self) -> Coordinate {
        *self
    }

    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::SingleId>, crate::Error> {
        Ok(std::iter::once(self.single_id(z)?))
    }

    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::RangeId>, crate::Error> {
        Ok(std::iter::once(RangeId::from(self.single_id(z)?)))
    }
}

impl Point for Coordinate {}
