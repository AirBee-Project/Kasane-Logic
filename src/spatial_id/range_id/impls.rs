use alloc::string::ToString;

use core::fmt;

use crate::{
    Coordinate, Error, RangeId, SpatialId, SpatialIdError, TemporalId,
    spatial_id::helpers::{self, format_dimension},
};
use core::str::FromStr;

impl fmt::Display for RangeId {
    /// `RangeId` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
    /// また、次元の範囲が単体の場合は自動的にその次元がSingle表示になります。
    ///
    /// 通常時の範囲表示
    /// ```no_run
    /// # use kasane_logic::RangeId;
    /// # use core::fmt::Write;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// Single範囲に自動圧縮（`f1=f2`）
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use core::fmt::Write;
    /// let id = RangeId::new(4, [-3,-3], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    ///  assert_eq!(s, "4/-3/8:9/5:10");;
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //空間の情報の書き込み
        write!(
            f,
            "{}/{}/{}/{}",
            self.z.get(),
            format_dimension(self.f),
            format_dimension(self.x),
            format_dimension(self.y),
        )?;

        //時間の情報があれば書き込み

        if !self.temporal_id.is_whole() {
            write!(f, "_{}", self.temporal_id)?;
        };
        Ok(())
    }
}

impl SpatialId for RangeId {
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

    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let min = self.f_min();
        let max = self.f_max();
        let z = self.z.get();

        let ns = self.f[0]
            .checked_add(by)
            .ok_or(SpatialIdError::FOutOfRange { f: i32::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(SpatialIdError::FOutOfRange { f: i32::MAX, z })?;

        if ns < min || ns > max {
            return Err(SpatialIdError::FOutOfRange { f: ns, z }.into());
        }
        if ne < min || ne > max {
            return Err(SpatialIdError::FOutOfRange { f: ne, z }.into());
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn move_x(&mut self, by: i32) {
        let max_len = self.x_max() as i64 + 1;
        self.x[0] = ((self.x[0] as i64 + by as i64).rem_euclid(max_len)) as u32;
        self.x[1] = ((self.x[1] as i64 + by as i64).rem_euclid(max_len)) as u32;
    }

    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        if by >= 0 {
            let byu = by as u32;
            let max = self.y_max();
            let z = self.z.get();

            let ns = self.y[0]
                .checked_add(byu)
                .ok_or(SpatialIdError::YOutOfRange { y: u32::MAX, z })?;
            let ne = self.y[1]
                .checked_add(byu)
                .ok_or(SpatialIdError::YOutOfRange { y: u32::MAX, z })?;

            if ns > max {
                return Err(SpatialIdError::YOutOfRange { y: ns, z }.into());
            }
            if ne > max {
                return Err(SpatialIdError::YOutOfRange { y: ne, z }.into());
            }

            self.y = [ns, ne];
            Ok(())
        } else {
            // south
            let byu = by.unsigned_abs();
            let max = self.y_max();
            let z = self.z.get();

            let ns = self.y[0]
                .checked_sub(byu)
                .ok_or(SpatialIdError::YOutOfRange { y: self.y_min(), z })?;
            let ne = self.y[1]
                .checked_sub(byu)
                .ok_or(SpatialIdError::YOutOfRange { y: self.y_min(), z })?;

            if ns > max {
                return Err(SpatialIdError::YOutOfRange { y: ns, z }.into());
            }
            if ne > max {
                return Err(SpatialIdError::YOutOfRange { y: ne, z }.into());
            }

            self.y = [ns, ne];
            Ok(())
        }
    }

    /// [`RangeId`] の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn spatial_center(&self) -> Coordinate {
        let z = self.z.get();

        let xf = (self.x[0] + self.x[1]) as f64 / 2.0 + 0.5;
        let yf = (self.y[0] + self.y[1]) as f64 / 2.0 + 0.5;
        let ff = (self.f[0] + self.f[1]) as f64 / 2.0 + 0.5;

        Coordinate::new(
            helpers::latitude(yf, z),
            helpers::longitude(xf, z),
            helpers::altitude(ff, z),
        )
        .unwrap()
    }

    /// [`RangeId`] の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn spatial_vertices(&self) -> [Coordinate; 8] {
        let z = self.z.get();

        // 2 点ずつの端点
        let xs = [self.x[0] as f64, (self.x[1] + 1) as f64];
        let ys = [self.y[0] as f64, (self.y[1] + 1) as f64];
        let fs = [self.f[0] as f64, (self.f[1] + 1) as f64];

        // 各軸方向の計算は 2 回だけにする
        let longitudes: [f64; 2] = [helpers::longitude(xs[0], z), helpers::longitude(xs[1], z)];

        let latitudes: [f64; 2] = [helpers::latitude(ys[0], z), helpers::latitude(ys[1], z)];

        let altitudes: [f64; 2] = [helpers::altitude(fs[0], z), helpers::altitude(fs[1], z)];

        let mut out = [Coordinate::default(); 8];

        let mut i = 0;
        for &altitude in &altitudes {
            for &latitude in &latitudes {
                for &longitude in &longitudes {
                    let _ = out[i].set_altitude(altitude);
                    let _ = out[i].set_latitude(latitude);
                    let _ = out[i].set_longitude(longitude);
                    i += 1;
                }
            }
        }

        out
    }

    ///その空間IDのＦ方向の長さをメートル単位で計算する関数
    fn length_f_meters(&self) -> f64 {
        //Z=25のとき、ちょうど高さが1mとなる
        let one = libm::pow(2_f64, (25 - self.z() as i32) as f64);

        //このRangeIdが表すセル数を計算（両端含む）
        let range = (self.f()[1] - self.f()[0] + 1) as f64;

        //かけ合わせて答えを返却
        one * range
    }

    ///その空間IDのX方向の長さをメートル単位で計算する関数
    fn length_x_meters(&self) -> f64 {
        //Todo:正確な実装ではないので将来的に置換
        let ecef: crate::Ecef = self.spatial_center().into();
        let r = libm::sqrt(ecef.x() * ecef.x() + ecef.y() * ecef.y());
        let one = r * 2.0 * core::f64::consts::PI / (libm::pow(2_f64, (self.z() as i32) as f64));
        let count = self.x()[0].abs_diff(self.x()[1]) as f64 + 1.0;

        one * count
    }

    ///その空間IDのY方向の長さをメートル単位で計算する関数
    fn length_y_meters(&self) -> f64 {
        //Todo:正確な実装ではないので将来的に置換
        let ecef: crate::Ecef = self.spatial_center().into();
        let r = libm::sqrt(ecef.x() * ecef.x() + ecef.y() * ecef.y());
        let one = r * 2.0 * core::f64::consts::PI / (libm::pow(2_f64, (self.z() as i32) as f64));
        let count = self.y()[0].abs_diff(self.y()[1]) as f64 + 1.0;

        one * count
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}

/// 文字列表現から [`RangeId`] を復元します。
///
/// 形式は [`Display`](core::fmt::Display) が出力する
/// `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
/// 単体範囲は `:` を省略した `"{z}/{f}/{x}/{y}"` 形式でもパース可能。
/// `temporal_id` feature が有効な場合は末尾の `_TemporalId` も受けつける。
///
/// ```
/// # use kasane_logic::RangeId;
/// let id: RangeId = "4/-3:6/8:9/5:10".parse().unwrap();
/// assert_eq!(id.z(), 4);
/// assert_eq!(id.f(), [-3, 6]);
/// assert_eq!(id.x(), [8, 9]);
/// assert_eq!(id.y(), [5, 10]);
/// ```
impl FromStr for RangeId {
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
        let f = parse_i32_dimension(f_text, s)?;
        let x = parse_u32_dimension(x_text, s)?;
        let y = parse_u32_dimension(y_text, s)?;

        #[cfg(feature = "temporal_id")]
        {
            let temporal_id = match temporal_text {
                Some(text) => TemporalId::from_str(text)?,
                None => TemporalId::WHOLE,
            };
            RangeId::new_with_temporal(z, f, x, y, temporal_id)
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            if temporal_text.is_some() {
                return Err(parse_error(s));
            }
            RangeId::new(z, f, x, y)
        }
    }
}

/// `"start:end"` または単体値の文字列を [`i32`] の範囲へ変換します。
fn parse_i32_dimension(text: &str, input: &str) -> Result<[i32; 2], Error> {
    match text.split_once(':') {
        Some((start, end)) => Ok([
            start.parse::<i32>().map_err(|_| parse_error(input))?,
            end.parse::<i32>().map_err(|_| parse_error(input))?,
        ]),
        None => Ok([text.parse::<i32>().map_err(|_| parse_error(input))?; 2]),
    }
}

/// `"start:end"` または単体値の文字列を [`u32`] の範囲へ変換します。
fn parse_u32_dimension(text: &str, input: &str) -> Result<[u32; 2], Error> {
    match text.split_once(':') {
        Some((start, end)) => Ok([
            start.parse::<u32>().map_err(|_| parse_error(input))?,
            end.parse::<u32>().map_err(|_| parse_error(input))?,
        ]),
        None => Ok([text.parse::<u32>().map_err(|_| parse_error(input))?; 2]),
    }
}

/// [`RangeId`] の文字列として解釈できない入力を表すエラーを生成します。
fn parse_error(input: &str) -> Error {
    SpatialIdError::ParseSpatialIdFormat {
        kind: "RangeId",
        input: input.to_string(),
    }
    .into()
}
