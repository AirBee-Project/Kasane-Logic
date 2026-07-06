use alloc::string::ToString;

use crate::{
    Coordinate, Ecef, Error, SingleId, SpatialId, SpatialIdError, TemporalId, spatial_id::helpers,
};
use core::fmt;
use core::str::FromStr;

impl fmt::Display for SingleId {
    /// `SingleId` を文字列形式で表示する。
    ///
    /// 形式は `"{z}/{f}/{x}/{y}"`。
    ///
    /// ```no_run
    /// # use kasane_logic::SingleId;
    /// # use core::fmt::Write;
    /// let id = SingleId::new(4, 6, 9, 10).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/6/9/10");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}/{}", self.z.get(), self.f, self.x, self.y)?;
        //時間の情報があれば書き込み
        if !self.temporal_id.is_whole() {
            write!(f, "_{}", self.temporal_id)?;
        }
        Ok(())
    }
}

impl SpatialId for SingleId {
    fn f_min(&self) -> i32 {
        self.z.f_min()
    }

    fn f_max(&self) -> i32 {
        self.z.f_max()
    }

    fn x_max(&self) -> u32 {
        self.z.xy_max()
    }

    fn y_max(&self) -> u32 {
        self.z.xy_max()
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleId` を垂直上下方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスが範囲外になる場合は[`SpatialIdError::FOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.f(), 6);
    ///
    /// let _ = id.move_f(-10).unwrap();
    /// assert_eq!(id.f(), -4);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::SpatialIdError;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.f(), 6);
    /// assert_eq!(id.move_f(50), Err(SpatialIdError::FOutOfRange { z: 4, f: 56 }.into()));
    /// ```
    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let new = self.f.checked_add(by).ok_or_else(|| {
            Error::from(SpatialIdError::FOutOfRange {
                f: if by >= 0 { i32::MAX } else { i32::MIN },
                z: self.z.get(),
            })
        })?;

        if new < self.f_min() || new > self.f_max() {
            return Err(SpatialIdError::FOutOfRange {
                f: new,
                z: self.z.get(),
            }
            .into());
        }

        self.f = new;

        Ok(())
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleId` を東西方向に動かします。WEBメルカトル図法において、東西方向は循環しているためどのような値を指定してもエラーは発生しません。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.x(), 9);
    ///
    /// let _ = id.move_x(-3);
    /// assert_eq!(id.x(), 6);
    /// ```
    ///
    /// 循環による移動
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.x(), 9);
    ///
    /// let _ = id.move_x(100);
    /// assert_eq!(id.x(), 13);
    /// ```
    fn move_x(&mut self, by: i32) {
        let max_len = self.x_max() as i64 + 1;
        let new = (self.x as i64 + by as i64).rem_euclid(max_len);
        self.x = new as u32;
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleId` を南北方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`SpatialIdError::YOutOfRange`]を返します
    ///
    /// 移動
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.y(), 10);
    ///
    /// let _ = id.move_y(-3).unwrap();
    /// assert_eq!(id.y(), 7);
    /// ```
    ///
    /// 範囲外の検知によるエラー
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::SpatialIdError;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.y(), 10);
    /// assert_eq!(id.move_y(-20), Err(SpatialIdError::YOutOfRange { z: 4, y: 0 }.into()));
    /// ```
    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        let new = if by >= 0 {
            self.y.checked_add(by as u32).ok_or_else(|| {
                Error::from(SpatialIdError::YOutOfRange {
                    y: u32::MAX,
                    z: self.z.get(),
                })
            })?
        } else {
            self.y
                .checked_sub(by.unsigned_abs())
                .ok_or(SpatialIdError::YOutOfRange {
                    y: self.y_min(),
                    z: self.z.get(),
                })?
        };

        if new > self.y_max() {
            return Err(SpatialIdError::YOutOfRange {
                y: new,
                z: self.z.get(),
            }
            .into());
        }

        self.y = new;

        Ok(())
    }

    /// `SingleId` の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Coordinate;
    /// let id = SingleId::new(4, 6, 9, 14).unwrap();
    /// let center: Coordinate = id.spatial_center();
    /// println!("{:?}", center);
    /// // Coordinate { latitude: -81.09321385260839, longitude: 33.75, altitude: 13631488.0 }
    /// ```
    fn spatial_center(&self) -> Coordinate {
        Coordinate::new(
            helpers::latitude(self.y as f64 + 0.5, self.z.get()),
            helpers::longitude(self.x as f64 + 0.5, self.z.get()),
            helpers::altitude(self.f as f64 + 0.5, self.z.get()),
        )
        .unwrap()
    }

    /// `SingleId` の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    ///
    /// ```
    /// # use kasane_logic::SpatialId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::Coordinate;
    /// let id = SingleId::new(4, 6, 9, 14).unwrap();
    /// let vertices: [Coordinate; 8] = id.spatial_vertices();
    /// println!("{:?}", vertices);
    ///
    ///  //[Coordinate { latitude: -79.17133464081945, longitude: 22.5, altitude: 12582912.0 }, Coordinate { latitude: -79.17133464081945, longitude: 45.0, altitude: 12582912.0 }, Coordinate { latitude: -82.67628497834903, longitude: 22.5, altitude: 12582912.0 }, Coordinate { latitude: -82.67628497834903, longitude: 45.0, altitude: 12582912.0 }, Coordinate { latitude: -79.17133464081945, longitude: 22.5, altitude: 14680064.0 }, Coordinate { latitude: -79.17133464081945, longitude: 45.0, altitude: 14680064.0 }, Coordinate { latitude: -82.67628497834903, longitude: 22.5, altitude: 14680064.0 }, Coordinate { latitude: -82.67628497834903, longitude: 45.0, altitude: 14680064.0 }]
    /// ```
    fn spatial_vertices(&self) -> [Coordinate; 8] {
        let xs = [self.x as f64, self.x as f64 + 1.0];
        let ys = [self.y as f64, self.y as f64 + 1.0];
        let fs = [self.f as f64, self.f as f64 + 1.0];

        // 各端点の値を前計算しておく
        let lon2 = [
            helpers::longitude(xs[0], self.z.get()),
            helpers::longitude(xs[1], self.z.get()),
        ];
        let lat2 = [
            helpers::latitude(ys[0], self.z.get()),
            helpers::latitude(ys[1], self.z.get()),
        ];
        let alt2 = [
            helpers::altitude(fs[0], self.z.get()),
            helpers::altitude(fs[1], self.z.get()),
        ];

        // 結果配列
        let mut out = [Coordinate::default(); 8];

        let mut i = 0;
        for &altitude in &alt2 {
            for &latitude in &lat2 {
                for &longitude in &lon2 {
                    out[i]
                        .set_longitude(longitude)
                        .expect("longitude must be within valid range");
                    out[i]
                        .set_latitude(latitude)
                        .expect("latitude must be within valid range");
                    out[i]
                        .set_altitude(altitude)
                        .expect("altitude must be within valid range");
                    i += 1;
                }
            }
        }

        out
    }

    ///その空間IDのＦ方向の長さをメートル単位で計算する関数
    fn length_f_meters(&self) -> f64 {
        //Z=25のとき、ちょうど高さが1mとなる
        libm::pow(2_f64, (25 - self.z() as i32) as f64)
    }

    ///その空間IDのX方向の長さをメートル単位で計算する関数
    fn length_x_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = libm::sqrt(ecef.x() * ecef.x() + ecef.y() * ecef.y());
        r * 2.0 * core::f64::consts::PI / ((1_u64 << self.z()) as f64)
    }

    ///その空間IDのY方向の長さをメートル単位で計算する関数
    fn length_y_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = libm::sqrt(ecef.x() * ecef.x() + ecef.y() * ecef.y());
        r * 2.0 * core::f64::consts::PI / ((1_u64 << self.z()) as f64)
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}

/// 文字列表現から [`SingleId`] を復元する。
///
/// 形式は [`Display`](core::fmt::Display) が出力する `"{z}/{f}/{x}/{y}"`
/// で、`temporal_id` feature が有効な場合のみ末尾に `_TemporalId` を付ける。
///
/// ```
/// # use kasane_logic::SingleId;
/// let id: SingleId = "5/3/2/10".parse().unwrap();
/// assert_eq!(id.z(), 5);
/// assert_eq!(id.f(), 3);
/// assert_eq!(id.x(), 2);
/// assert_eq!(id.y(), 10);
/// ```
impl FromStr for SingleId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (body, temporal_text) = match s.split_once('_') {
            Some((body, temporal_text)) => (body, Some(temporal_text)),
            None => (s, None),
        };

        let mut parts = body.split('/');
        let z_text = parts.next().ok_or_else(|| parse_error(s))?;
        let f_text = parts.next().ok_or_else(|| parse_error(s))?;
        let x_text = parts.next().ok_or_else(|| parse_error(s))?;
        let y_text = parts.next().ok_or_else(|| parse_error(s))?;
        if parts.next().is_some() {
            return Err(parse_error(s));
        }

        let z = z_text.parse::<u8>().map_err(|_| parse_error(s))?;
        let f = f_text.parse::<i32>().map_err(|_| parse_error(s))?;
        let x = x_text.parse::<u32>().map_err(|_| parse_error(s))?;
        let y = y_text.parse::<u32>().map_err(|_| parse_error(s))?;

        #[cfg(feature = "temporal_id")]
        {
            let temporal_id = match temporal_text {
                Some(text) => TemporalId::from_str(text)?,
                None => TemporalId::WHOLE,
            };
            SingleId::new(z, f, x, y).map(|id| id.with_temporal(temporal_id))
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            if temporal_text.is_some() {
                return Err(parse_error(s));
            }
            SingleId::new(z, f, x, y)
        }
    }
}

/// [`SingleId`] の文字列表現として解釈できないことを表すエラーを生成します。
fn parse_error(input: &str) -> Error {
    SpatialIdError::ParseSpatialIdFormat {
        kind: "SingleId",
        input: input.to_string(),
    }
    .into()
}
