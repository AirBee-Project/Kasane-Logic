use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{Coordinate, FlexId, SingleId, TemporalId, error::Error};

#[cfg(doc)]
use crate::RangeId;

/// [SingleId],[RangeId],[FlexId]が共通して持つTrait
pub trait SpatialId:
    IntoFlexIds
    + IterFlexIds
    + IterSingleIds
    + IntoSingleIds
    + Debug
    + Display
    + Clone
    + Eq
    + Hash
    + Ord
    + PartialOrd
    + FromStr
{
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
    fn temporal(&self) -> &TemporalId;

    /// 時間 ID を可変参照で返す。
    fn temporal_mut(&mut self) -> &mut TemporalId;
}

/// [SingleId] の集合であることを保証するトレイト。
pub trait IntoSingleIds {
    type IntoIter: Iterator<Item = SingleId>;

    /// 所有権ごと [SingleId] の列へ変換する。
    fn into_single_ids(self) -> Self::IntoIter;
}

/// [SingleId] の参照列を返せることを保証するトレイト。
pub trait IterSingleIds {
    type Iter<'a>: Iterator<Item = SingleId> + 'a
    where
        Self: 'a;

    /// 参照から [SingleId] の列を列挙する。
    fn iter_single_ids(&self) -> Self::Iter<'_>;
}

/// [FlexId] の集合であることを保証するトレイト。
pub trait IntoFlexIds {
    type IntoIter: Iterator<Item = FlexId>;

    /// 所有権ごと [FlexId] の列へ変換する。
    fn into_flex_ids(self) -> Self::IntoIter;
}

/// [FlexId] の参照列を返せることを保証するトレイト。
pub trait IterFlexIds {
    type Iter<'a>: Iterator<Item = FlexId> + 'a
    where
        Self: 'a;

    /// 参照から [FlexId] の列を列挙する。
    fn iter_flex_ids(&self) -> Self::Iter<'_>;
}
