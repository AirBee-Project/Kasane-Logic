use crate::{
    Block, Coordinate, Ecef, Error, F_MAX, F_MIN, Segment, SingleId, SpatialId, TemporalId, XY_MAX,
    spatial_id::{BlockSegments, helpers},
};
use std::fmt;

impl fmt::Display for SingleId {
    /// `SingleId` を文字列形式で表示する。
    ///
    /// 形式は `"{z}/{f}/{x}/{y}"`。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use std::fmt::Write;
    /// let id = SingleId::new(4, 6, 9, 10).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "4/6/9/10");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}/{}", self.z, self.f, self.x, self.y)?;
        //時間の情報があれば書き込み
        if !self.temporal_id.is_whole() {
            write!(f, "_{}", self.temporal_id)?;
        }
        Ok(())
    }
}

impl SpatialId for SingleId {
    /// このIDのズームレベルにおける最小の F インデックスを返す
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.min_f(), -32i32);
    /// ```
    fn min_f(&self) -> i32 {
        F_MIN[self.z as usize]
    }

    /// このIDのズームレベルにおける最大の F インデックスを返す
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.max_f(), 31i32);
    /// ```
    fn max_f(&self) -> i32 {
        F_MAX[self.z as usize]
    }

    /// このIDのズームレベルにおける最大の XY インデックスを返す
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.max_xy(), 31u32);
    /// ```
    fn max_xy(&self) -> u32 {
        XY_MAX[self.z as usize]
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleId` を垂直上下方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Fインデックスが範囲外になる場合は[`Error::FOutOfRange`]を返します
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
    /// # use kasane_logic::Error;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.f(), 6);
    /// assert_eq!(id.move_f(50), Err(Error::FOutOfRange { z: 4, f: 56 }));
    /// ```
    fn move_f(&mut self, by: i32) -> Result<(), Error> {
        let new = self.f.checked_add(by).ok_or(Error::FOutOfRange {
            f: if by >= 0 { i32::MAX } else { i32::MIN },
            z: self.z,
        })?;

        if new < self.min_f() || new > self.max_f() {
            return Err(Error::FOutOfRange { f: new, z: self.z });
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
    /// assert_eq!(id.x(), 4);
    /// ```
    fn move_x(&mut self, by: i32) {
        let new = (self.x as i32 + by).rem_euclid(self.max_xy().try_into().unwrap());
        self.x = new as u32;
    }

    /// 指定したインデックス差 `by` に基づき、この `SingleId` を南北方向に動かします。
    ///
    /// # パラメータ
    /// * `by` — インデックス差
    ///
    /// # バリデーション
    /// - Yインデックスが範囲外になる場合は[`Error::YOutOfRange`]を返します
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
    /// # use kasane_logic::Error;
    /// let mut id = SingleId::new(4, 6, 9, 10).unwrap();
    /// assert_eq!(id.y(), 10);
    /// assert_eq!(id.move_y(-20), Err(Error::YOutOfRange { z: 4, y: 0 }));
    /// ```
    fn move_y(&mut self, by: i32) -> Result<(), Error> {
        let new = if by >= 0 {
            self.y.checked_add(by as u32).ok_or(Error::YOutOfRange {
                y: u32::MAX,
                z: self.z,
            })?
        } else {
            self.y
                .checked_sub(-by as u32)
                .ok_or(Error::YOutOfRange { y: 0, z: self.z })?
        };

        if new > self.max_xy() {
            return Err(Error::YOutOfRange { y: new, z: self.z });
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
        unsafe {
            Coordinate::new_unchecked(
                helpers::latitude(self.y as f64 + 0.5, self.z),
                helpers::longitude(self.x as f64 + 0.5, self.z),
                helpers::altitude(self.f as f64 + 0.5, self.z),
            )
        }
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

        // 各端点の値を前計算しておく（計算コスト削減）
        let lon2 = [
            helpers::longitude(xs[0], self.z),
            helpers::longitude(xs[1], self.z),
        ];
        let lat2 = [
            helpers::latitude(ys[0], self.z),
            helpers::latitude(ys[1], self.z),
        ];
        let alt2 = [
            helpers::altitude(fs[0], self.z),
            helpers::altitude(fs[1], self.z),
        ];

        // 結果配列（Default を利用）
        let mut out = [Coordinate::default(); 8];

        let mut i = 0;
        for f_i in 0..2 {
            for y_i in 0..2 {
                for x_i in 0..2 {
                    out[i]
                        .set_longitude(lon2[x_i])
                        .expect("longitude must be within valid range");
                    out[i]
                        .set_latitude(lat2[y_i])
                        .expect("latitude must be within valid range");
                    out[i]
                        .set_altitude(alt2[f_i])
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
        2_i32.pow(25 - self.z() as u32) as f64
    }

    ///その空間IDのX方向の長さをメートル単位で計算する関数
    fn length_x_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(self.z() as u32) as f64)
    }

    ///その空間IDのY方向の長さをメートル単位で計算する関数
    fn length_y_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(self.z() as u32) as f64)
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}

impl Block for SingleId {
    fn segmentation(&self) -> BlockSegments {
        let f_segment = Segment::from_f(self.z(), self.f());
        let x_segment = Segment::from_xy(self.z(), self.x());
        let y_segment = Segment::from_xy(self.z(), self.y());

        BlockSegments {
            f: vec![f_segment],
            x: vec![x_segment],
            y: vec![y_segment],
        }
    }
}
