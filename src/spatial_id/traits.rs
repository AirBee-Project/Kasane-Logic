use core::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{Coordinate, FlexId, RangeId, error::Error};

#[cfg(doc)]
use crate::SingleId;

/// 時間区間を表す型が共通して持つTrait。
///
/// [`FlexId`]/[`SingleId`]（点）は生の2進セル表現（`TemporalCell`）を、[`RangeId`]（範囲）は
/// 人間に読みやすい範囲表現（`TemporalRange`）を持つ。両者の形は異なるが、[`SpatialId::temporal`]
/// を通じて共通の操作（全時間判定・絶対秒区間への変換）を行えるようにするためのTrait。
pub trait TemporalId:
    Debug + Display + Clone + Eq + Hash + Ord + PartialOrd + FromStr<Err = Error>
{
    /// 全時間を表す値。
    const WHOLE: Self;

    /// このインスタンスが全時間を表す特別な値（[`WHOLE`](Self::WHOLE)）であるかを判定する。
    fn is_whole(&self) -> bool;

    /// この値が表す絶対秒区間 `[start, end)` を返す。
    ///
    /// 形の異なる2つの具象型（生セル・人間向け範囲）を横断して比較・集約するための共通基盤。
    fn seconds_range(&self) -> (u64, u64);
}

/// [SingleId],[RangeId],[FlexId]が共通して持つTrait
pub trait SpatialId:
    IntoIterator<Item = FlexId>
    + Debug
    + Display
    + Clone
    + Eq
    + Hash
    + Ord
    + PartialOrd
    + FromStr
    + Into<RangeId>
{
    /// この型が保持する時間区間の具象表現。
    ///
    /// [`FlexId`]/[`SingleId`]は生の2進セル（`TemporalCell`）、[`RangeId`]は人間に読みやすい
    /// 範囲（`TemporalRange`）を持つため、型ごとに異なりうる。
    type Temporal: TemporalId;
    /// ズームレベルにおける最小のFインデックスを返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.f_min(),-8);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.f_min(),-16);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.f_min(),-32);
    /// ```
    fn f_min(&self) -> i32;

    /// ズームレベルにおける最大のFインデックスを返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.f_max(),7);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.f_max(),15);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.f_max(),31);
    /// ```
    fn f_max(&self) -> i32;

    /// ズームレベルにおける最大のXインデックスを返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.x_max(),7);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.x_max(),15);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.x_max(),3);
    /// ```
    fn x_max(&self) -> u32;

    /// ズームレベルにおける最小のXインデックスを返す。全てのIDにおいて必ず`0`を返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.x_min(),0);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.x_min(),0);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.x_min(),0);
    /// ```
    fn x_min(&self) -> u32 {
        0
    }

    /// ズームレベルにおける最大のYインデックスを返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.y_max(),7);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.y_max(),15);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.y_max(),1023);
    /// ```
    fn y_max(&self) -> u32;

    /// ズームレベルにおける最小のYインデックスを返す。全てのIDにおいて必ず`0`を返す。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::FlexId;
    /// # use kasane_logic::SpatialId;
    /// //SingleIdの動作
    /// let single_id=SingleId::new(3,3,2,4).unwrap();
    /// assert_eq!(single_id.y_min(),0);
    ///
    /// //RangeIdの動作
    /// let range_id=RangeId::new(4, [-3,10], [8,9], [5,10]).unwrap();
    /// assert_eq!(range_id.y_min(),0);
    ///
    /// //FlexIdの動作
    /// let flex_id=FlexId::new(5, 3, 2, 3, 10, 1).unwrap();
    /// assert_eq!(flex_id.y_min(),0);
    /// ```
    fn y_min(&self) -> u32 {
        0
    }

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

    /// 時間 ID を参照で返す。
    fn temporal(&self) -> &Self::Temporal;

    /// 時間 ID を可変参照で返す。
    fn temporal_mut(&mut self) -> &mut Self::Temporal;
}
