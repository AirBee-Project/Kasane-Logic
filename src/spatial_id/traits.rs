use core::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{Coordinate, FlexId, Interval, RangeId, error::Error};

#[cfg(doc)]
use crate::SingleId;

/// [SingleId],[RangeId],[FlexId]が共通して持つTrait
pub trait SpatialId:
    IntoIterator<Item = FlexId>
    + Into<RangeId>
    + Debug
    + Display
    + Clone
    + Eq
    + Hash
    + Ord
    + PartialOrd
    + FromStr
{
    /// F 方向に指定インデックスだけ移動する。
    fn move_f(&mut self, by: i32) -> Result<(), Error>;

    /// X 方向に指定インデックスだけ移動する。
    fn move_x(&mut self, by: i32);

    /// Y 方向に指定インデックスだけ移動する。
    fn move_y(&mut self, by: i32) -> Result<(), Error>;

    /// F 方向の長さをメートル単位で返す。
    fn length_f_meters(&self) -> f64;

    /// X 方向の長さをメートル単位で返す。
    fn length_x_meters(&self) -> f64;

    /// Y 方向の長さをメートル単位で返す。
    fn length_y_meters(&self) -> f64;

    /// 空間 ID の中心座標を返す。
    fn spatial_center(&self) -> Coordinate;

    /// 空間 ID の8頂点を返す。
    fn spatial_vertices(&self) -> [Coordinate; 8];

    /// 時間間隔 `{i}`を返す。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, SingleId, SpatialId};
    /// let id = SingleId::new(12, 0, 3638, 1614).unwrap().with_time(1800, 809712).unwrap();
    /// assert_eq!(id.time_interval().seconds(), 1800);
    ///
    /// let plain = SingleId::new(12, 0, 3638, 1614).unwrap();
    /// assert_eq!(plain.time_interval(), Interval::WHOLE);
    /// # }
    /// ```
    fn time_interval(&self) -> Interval;

    /// 占有する絶対秒区間 `[start, end)` を返す。
    fn seconds_range(&self) -> (u64, u64);

    /// 占有する絶対秒区間を [`TimeSpan`](crate::spatial_id::time::span::TimeSpan) として返す。
    fn time_span(&self) -> crate::spatial_id::time::span::TimeSpan {
        let (s, e) = self.seconds_range();
        crate::spatial_id::time::span::TimeSpan::new_unchecked(s, e)
    }

    /// 時間を指定していない（全時間を覆う）かを判定する。
    ///
    /// ```
    /// # use kasane_logic::{SingleId, SpatialId};
    /// assert!(SingleId::new(12, 0, 3638, 1614).unwrap().is_whole_time());
    /// ```
    fn is_whole_time(&self) -> bool {
        self.seconds_range() == (0, Interval::MAX_SECONDS)
    }
}
