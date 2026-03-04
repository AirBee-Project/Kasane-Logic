use std::fmt;

use crate::{
    Block, Coordinate, Error, RangeId, Segment, SingleId, SpatialId, TemporalId,
    spatial_id::{
        BlockSegments,
        constants::{F_MAX, F_MIN, XY_MAX},
        helpers,
    },
};

impl fmt::Display for RangeId {
    /// `RangeId` を文字列形式で表示します。
    ///
    /// 形式は `"{z}/{f1}:{f2}/{x1}:{x2}/{y1}:{y2}"` です。
    /// また、次元の範囲が単体の場合は自動的にその次元がSingle表示になります。
    ///
    /// 通常時の範囲表示
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use std::fmt::Write;
    /// let id = RangeId::new(4, [-3,6], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/-3:6/8:9/5:10");
    /// ```
    ///
    /// Single範囲に自動圧縮（`f1=f2`）
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use std::fmt::Write;
    /// let id = RangeId::new(4, [-3,-3], [8,9], [5,10]).unwrap();
    /// let s = format!("{}", id);
    ///  assert_eq!(s, "4/-3/8:9/5:10");;
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //空間の情報の書き込み
        write!(
            f,
            "{}/{}/{}/{}",
            self.z,
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

//次元の文字列を圧縮するための関数
fn format_dimension<T: PartialEq + fmt::Display>(dimension: [T; 2]) -> String {
    if dimension[0] == dimension[1] {
        format!("{}", dimension[0])
    } else {
        format!("{}:{}", dimension[0], dimension[1])
    }
}

impl SpatialId for RangeId {
    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// # use kasane_logic::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.min_f(), -32i32);
    /// ```
    fn min_f(&self) -> i32 {
        F_MIN[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// # use kasane_logic::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.max_f(), 31i32);
    /// ```
    fn max_f(&self) -> i32 {
        F_MAX[self.z as usize]
    }

    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// # use kasane_logic::SpatialId;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.max_xy(), 31u32);
    /// ```
    fn max_xy(&self) -> u32 {
        XY_MAX[self.z as usize]
    }

    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        let z = self.z;

        let ns = self.f[0]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i32::MAX, z })?;
        let ne = self.f[1]
            .checked_add(by)
            .ok_or(Error::FOutOfRange { f: i32::MAX, z })?;

        if ns < min || ns > max {
            return Err(Error::FOutOfRange { f: ns, z });
        }
        if ne < min || ne > max {
            return Err(Error::FOutOfRange { f: ne, z });
        }

        self.f = [ns, ne];
        Ok(())
    }

    fn move_x(&mut self, by: i32) {
        let new = (self.x[0] as i32 + by).rem_euclid(self.max_xy().try_into().unwrap());
        self.x[0] = new as u32;

        let new = (self.x[1] as i32 + by).rem_euclid(self.max_xy().try_into().unwrap());
        self.x[1] = new as u32;
    }

    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        if by >= 0 {
            let byu = by as u32;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.y[0]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u32::MAX, z })?;
            let ne = self.y[1]
                .checked_add(byu)
                .ok_or(Error::YOutOfRange { y: u32::MAX, z })?;

            if ns > max {
                return Err(Error::YOutOfRange { y: ns, z });
            }
            if ne > max {
                return Err(Error::YOutOfRange { y: ne, z });
            }

            self.y = [ns, ne];
            Ok(())
        } else {
            // south
            let byu = (-by) as u32;
            let max = self.max_xy();
            let z = self.z;

            let ns = self.y[0]
                .checked_sub(byu)
                .ok_or(Error::YOutOfRange { y: 0, z })?;
            let ne = self.y[1]
                .checked_sub(byu)
                .ok_or(Error::YOutOfRange { y: 0, z })?;

            if ns > max {
                return Err(Error::YOutOfRange { y: ns, z });
            }
            if ne > max {
                return Err(Error::YOutOfRange { y: ne, z });
            }

            self.y = [ns, ne];
            Ok(())
        }
    }

    /// [`RangeId`] の中心座標を[`Coordinate`]型で返します。
    ///
    /// 中心座標は空間IDの最も外側の頂点の8点の平均座標です。現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn spatial_center(&self) -> Coordinate {
        let z = self.z;

        let xf = (self.x[0] + self.x[1]) as f64 / 2.0 + 0.5;
        let yf = (self.y[0] + self.y[1]) as f64 / 2.0 + 0.5;
        let ff = (self.f[0] + self.f[1]) as f64 / 2.0 + 0.5;

        unsafe {
            Coordinate::new_unchecked(
                helpers::longitude(xf, z),
                helpers::latitude(yf, z),
                helpers::altitude(ff, z),
            )
        }
    }

    /// [`RangeId`] の最も外側の頂点の8点の座標を[`Coordinate`]型の配列として返します。
    ///
    /// 現実空間における空間IDは完全な直方体ではなく、緯度や高度によって歪みが発生していることに注意する必要があります。
    fn spatial_vertices(&self) -> [Coordinate; 8] {
        let z = self.z;

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
        for fi in 0..2 {
            for yi in 0..2 {
                for xi in 0..2 {
                    let _ = out[i].set_altitude(altitudes[fi]);
                    let _ = out[i].set_latitude(latitudes[yi]);
                    let _ = out[i].set_longitude(longitudes[xi]);
                    i += 1;
                }
            }
        }

        out
    }

    ///その空間IDのＦ方向の長さをメートル単位で計算する関数
    fn length_f_meters(&self) -> f64 {
        //Z=25のとき、ちょうど高さが1mとなる
        let one = 2_i32.pow(25 - self.z() as u32) as f64;

        //このRangeIdの高さ方向の幅を計算
        let range = (self.f()[0] - self.f()[1]).abs() as f64;

        //かけ合わせて答えを返却
        (one * range).into()
    }

    ///その空間IDのX方向の長さをメートル単位で計算する関数
    fn length_x_meters(&self) -> f64 {
        todo!()
    }

    ///その空間IDのY方向の長さをメートル単位で計算する関数
    fn length_y_meters(&self) -> f64 {
        todo!()
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}

impl Block for RangeId {
    fn segmentation(&self) -> BlockSegments {
        let f = Segment::split_f(self.z(), self.f()).collect();
        let x = if self.x[0] <= self.x[1] {
            Segment::split_xy(self.z(), self.x()).collect()
        } else {
            Segment::split_xy(self.z(), [self.x[0], self.max_xy()])
                .chain(Segment::split_xy(self.z(), [0, self.x[1]]))
                .collect()
        };
        let y = Segment::split_xy(self.z(), self.y()).collect();
        BlockSegments { f, x, y }
    }
}

impl From<SingleId> for RangeId {
    ///`SingleId`を[`RangeId`]に変換します。表す物理的な範囲に変化はありません。
    fn from(id: SingleId) -> Self {
        RangeId {
            z: id.z(),
            f: [id.f(), id.f()],
            x: [id.x(), id.x()],
            y: [id.y(), id.y()],
            temporal_id: id.temporal().clone(),
        }
    }
}
