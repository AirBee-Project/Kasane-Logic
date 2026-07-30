pub mod constructor;
pub mod convert;
pub mod impls;
pub mod random;

use crate::{
    Interval, SpatialIdError,
    error::Error,
    spatial_id::{helpers, time_cells, zoom_level::ZoomLevel},
};

/// RangeIdは空間IDの範囲表現を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// # use kasane_logic::ZoomLevel;
/// pub struct RangeId {
///     z: ZoomLevel,
///     f: [i32; 2],
///     x: [u32; 2],
///     y: [u32; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeId {
    z: ZoomLevel,
    f: [i32; 2],
    x: [u32; 2],
    y: [u32; 2],
    i: Interval,
    t: [u64; 2],
}

impl RangeId {
    /// この `RangeId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z.get()
    }

    /// この `RangeId` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.f(), [-3i32,29i32]);
    /// ```
    pub fn f(&self) -> [i32; 2] {
        self.f
    }

    /// この `RangeId` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.x(), [8u32,9u32]);
    /// ```
    pub fn x(&self) -> [u32; 2] {
        self.x
    }

    /// この `RangeId` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.y(), [5u32,10u32]);
    /// ```
    pub fn y(&self) -> [u32; 2] {
        self.y
    }

    /// この `RangeId` の時間間隔 `{i}`（単位、秒数）を返します。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId};
    /// let id = RangeId::new(4, [-3, 6], [8, 9], [5, 10]).unwrap()
    ///     .with_time(Interval::HOUR, [5, 8]).unwrap();
    /// assert_eq!(id.interval(), Interval::HOUR);
    /// assert_eq!(id.t(), [5, 8]);
    /// # }
    /// ```
    pub fn interval(&self) -> Interval {
        self.i
    }

    /// この `RangeId` の時間インデックス範囲 `[min, max]`（両端含む）を返します。
    pub fn t(&self) -> [u64; 2] {
        self.t
    }

    /// この `RangeId` が占める絶対秒区間 `[start, end)` を返します（1970-01-01 00:00 UTC 起点）。
    pub fn seconds_range(&self) -> (u64, u64) {
        let unit = self.i.seconds();
        (self.t[0] * unit, (self.t[1] + 1) * unit)
    }

    /// 時間を設定した自身を返します（ビルダー形式）。
    ///
    /// [`SingleId::with_time`](crate::SingleId::with_time)が単一セルしか受け取らないのに対し、
    /// こちらは空間と同じく**範囲**を受け取る。`t` は `[min, max]` の `[u64; 2]`、または
    /// 両端が等しい単一の `u64` のどちらでも渡せる（f/x/y 引数と同じ考え方）。
    ///
    /// FlexTreeが必要とする2の冪秒のセルへの分解は、木へ挿入する段階
    /// （[`IntoIterator`]による[`FlexId`](crate::FlexId)への展開）で自動的に行われる。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId};
    /// let id = RangeId::new(4, [-3, 6], [8, 9], [5, 10]).unwrap()
    ///     .with_time(Interval::HOUR, [5, 8]).unwrap();
    /// assert_eq!(id.to_string(), "4/-3:6/8:9/5:10_3600/5:8");
    ///
    /// // 単一の時刻も渡せる。
    /// let single = RangeId::new(4, [-3, 6], [8, 9], [5, 10]).unwrap()
    ///     .with_time(Interval::HOUR, 5).unwrap();
    /// assert_eq!(single.to_string(), "4/-3:6/8:9/5:10_3600/5");
    /// # }
    /// ```
    pub fn with_time(
        mut self,
        interval: impl Into<i64>,
        t: impl helpers::IntoRange<u64>,
    ) -> Result<Self, Error> {
        let mut t = t.into_range();
        if t[0] > t[1] {
            t.swap(0, 1);
        }

        let interval = Interval::from_signed_seconds(interval.into())?;
        let unit = interval.seconds();
        let end_exclusive = t[1]
            .checked_add(1)
            .and_then(|v| v.checked_mul(unit))
            .ok_or(SpatialIdError::TOutOfRange { i: unit, t: t[1] })?;

        if end_exclusive > Interval::WHOLE_SECONDS {
            return Err(SpatialIdError::TOutOfRange { i: unit, t: t[1] }.into());
        }

        self.i = interval;
        self.t = t;
        Ok(self)
    }

    /// Unix 時刻（秒）の区間 `[start, end)` を覆う時間を設定した自身を返します。
    ///
    /// 単位は「その区間をちょうど表せる最も粗い秒数」が自動で選ばれる
    /// （`start` と区間幅の最大公約数）。仕様が認める任意秒数の単位もそのまま出てくる。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::RangeId;
    /// // [1457481600, 1457483400) はちょうど 1800 秒ぶん。
    /// let id = RangeId::new(4, 0, 0, 0).unwrap()
    ///     .with_time_seconds(1_457_481_600, 1_457_483_400).unwrap();
    /// assert_eq!(id.interval().seconds(), 1800);
    /// assert_eq!(id.t(), [809712, 809712]);
    /// # }
    /// ```
    pub fn with_time_seconds(mut self, start: u64, end: u64) -> Result<Self, Error> {
        if start >= end || end > Interval::WHOLE_SECONDS {
            return Err(SpatialIdError::TOutOfRange { i: 1, t: end }.into());
        }

        // その区間をちょうど表せる最も粗い単位を選ぶ（`gcd(start, 幅)`）。
        let unit = time_cells::coarsest_unit(start, end - start);
        self.i = Interval::from_seconds_unchecked(unit);
        self.t = [start / unit, end / unit - 1];
        Ok(self)
    }

    /// 同じ絶対秒区間を、別の単位で表し直した自身を返します。割り切れなければ [`None`]。
    pub fn relabel_time(mut self, interval: Interval) -> Option<Self> {
        let (start, end) = self.seconds_range();
        let unit = interval.seconds();
        if !start.is_multiple_of(unit) || !end.is_multiple_of(unit) {
            return None;
        }
        self.i = interval;
        self.t = [start / unit, end / unit - 1];
        Some(self)
    }

    /// この `RangeId` の時間を、FlexTree が使う2進セルの列へ分解する。クレート内部専用。
    pub(crate) fn time_cells(&self) -> time_cells::TimeCells {
        let (start, end) = self.seconds_range();
        time_cells::split_seconds(start, end)
    }

    pub fn set_f(&mut self, value: [i32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let f_min = ZoomLevel::new(z).unwrap().f_min();
        let f_max = ZoomLevel::new(z).unwrap().f_max();

        for &f_value in &value {
            if f_value < f_min || f_value > f_max {
                return Err(SpatialIdError::FOutOfRange { f: f_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.f = value;
        Ok(())
    }

    pub fn set_x(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let xy_max = ZoomLevel::new(z).unwrap().xy_max();

        for &x_value in &value {
            if x_value > xy_max {
                return Err(SpatialIdError::XOutOfRange { x: x_value, z }.into());
            }
        }

        self.x = value;
        Ok(())
    }

    pub fn set_y(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let xy_max = ZoomLevel::new(z).unwrap().xy_max();

        for &y_value in &value {
            if y_value > xy_max {
                return Err(SpatialIdError::YOutOfRange { y: y_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル `target_z` に細分化した、この `RangeId` を含むすべての子 `RangeId` を生成します。
    ///
    /// # パラメータ
    /// * `target_z` — 生成したい子 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより浅い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段深いズームへの細分化
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(6).unwrap();
    /// assert_eq!(result,  RangeId::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// 現在より浅いズームを指定した場合
    /// ```
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(4);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 4 }))));
    /// ```
    pub fn spatial_children_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z < z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let difference = target_z - z;
        let scale_f = 1_i32 << difference as u32;
        let scale_xy = 1_u32 << difference as u32;

        let f = helpers::scale_range_i32(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u32(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u32(self.y[0], self.y[1], scale_xy);

        Ok(RangeId {
            z: ZoomLevel::new(target_z).unwrap(),
            f,
            x,
            y,

            i: self.i,
            t: self.t,
        })
    }

    /// 指定したズームレベル `target_z` に縮約した、この `RangeId` の親 `RangeId` を返します。
    ///
    /// # パラメータ
    /// * `target_z` — 取得したい親 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより深い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段浅いズームへの縮約
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [1,29], [8,9], [5,10]).unwrap();
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [0,14]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// Fが負の場合の挙動:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [-5,-3]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// 現在より深いズームを指定した場合:
    /// ```
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_parent_at_zoom(6);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 6 }))));
    /// ```
    pub fn spatial_parent_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z > z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let shift = (z - target_z) as u32;

        let f = [
            if self.f[0] == -1 {
                -1
            } else {
                self.f[0] >> shift
            },
            if self.f[1] == -1 {
                -1
            } else {
                self.f[1] >> shift
            },
        ];

        let x = [self.x[0] >> shift, self.x[1] >> shift];
        let y = [self.y[0] >> shift, self.y[1] >> shift];

        Ok(RangeId {
            z: ZoomLevel::new(target_z).unwrap(),
            f,
            x,
            y,

            i: self.i,
            t: self.t,
        })
    }
}
