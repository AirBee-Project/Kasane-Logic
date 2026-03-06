use std::fmt;

use crate::{
    Block, Coordinate, Error, RangeId, Segment, SingleId, SpatialId, TemporalId,
    spatial_id::{
        BlockSegments,
        constants::{F_MAX, F_MIN, XY_MAX},
        helpers::{self, format_dimension},
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

    fn single_ids(&self) -> impl Iterator<Item = SingleId> {
        let z = self.z;

        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];

        f_range.flat_map(move |f| {
            let y_range = y_range.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=self.max_xy())
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                y_range
                    .clone()
                    .map(move |y| unsafe { SingleId::new_unchecked(z, f, x, y) })
            })
        })
    }

    fn optimize_single_ids(&self) -> impl Iterator<Item = SingleId> {
        let mut regions = Vec::new();
        let (f, x, y) = (self.f(), self.x(), self.y());

        let mut stack = if x[0] <= x[1] {
            vec![(self.z(), f[0], f[1], x[0], x[1], y[0], y[1])]
        } else {
            vec![
                (self.z(), f[0], f[1], x[0], self.max_xy(), y[0], y[1]),
                (self.z(), f[0], f[1], 0, x[1], y[0], y[1]),
            ]
        };

        while let Some((z, f0, f1, x0, x1, y0, y1)) = stack.pop() {
            if f0 > f1 || x0 > x1 || y0 > y1 {
                continue;
            }

            if z == 0 {
                regions.push((z, f0, f1, x0, x1, y0, y1));
                continue;
            }

            let split_u = |s: u32, e: u32| {
                let (i_s, i_e) = (s + s % 2, if e % 2 == 1 { e + 1 } else { e });
                if i_s < i_e {
                    [s..i_s, i_s..i_e, i_e..e + 1]
                } else {
                    [s..e + 1, 0..0, 0..0]
                }
            };
            let split_i = |s: i32, e: i32| {
                let (i_s, i_e) = (
                    s + s.rem_euclid(2),
                    if e.rem_euclid(2) == 1 { e + 1 } else { e },
                );
                if i_s < i_e {
                    [s..i_s, i_s..i_e, i_e..e + 1]
                } else {
                    [s..e + 1, 0..0, 0..0]
                }
            };

            let (fp, xp, yp) = (split_i(f0, f1), split_u(x0, x1), split_u(y0, y1));

            for (i, fr) in fp.iter().enumerate() {
                for (j, xr) in xp.iter().enumerate() {
                    for (k, yr) in yp.iter().enumerate() {
                        if i == 1 && j == 1 && k == 1 {
                            continue;
                        }

                        // Rangeが空でなければ、出力すべき直方体領域として記録
                        if !fr.is_empty() && !xr.is_empty() && !yr.is_empty() {
                            regions.push((
                                z,
                                fr.start,
                                fr.end - 1,
                                xr.start,
                                xr.end - 1,
                                yr.start,
                                yr.end - 1,
                            ));
                        }
                    }
                }
            }

            if !fp[1].is_empty() && !xp[1].is_empty() && !yp[1].is_empty() {
                stack.push((
                    z - 1,
                    fp[1].start >> 1,
                    (fp[1].end - 1) >> 1,
                    xp[1].start >> 1,
                    (xp[1].end - 1) >> 1,
                    yp[1].start >> 1,
                    (yp[1].end - 1) >> 1,
                ));
            }
        }

        regions.into_iter().flat_map(|(z, f0, f1, x0, x1, y0, y1)| {
            (f0..=f1).flat_map(move |f| {
                (x0..=x1).flat_map(move |x| {
                    (y0..=y1).map(move |y| unsafe { SingleId::new_unchecked(z, f, x, y) })
                })
            })
        })
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
