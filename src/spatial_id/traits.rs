use core::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use crate::{Coordinate, FlexId, RangeId, TemporalId, error::Error};

#[cfg(doc)]
use crate::SingleId;

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

    /// 時間 ID を返す。
    ///
    /// [`FlexId`]/[`SingleId`]（点）は内部の生の2進セル表現から、[`RangeId`]（範囲）は保持している
    /// 値からそれぞれ変換・複製して返す。返す型は3つの実装型で共通の[`TemporalId`]。
    fn temporal(&self) -> TemporalId;

    /// 時間 ID を設定した自身を返す。
    ///
    /// # バリデーション
    /// [`FlexId`]/[`SingleId`]（点）は与えられた`temporal`がFlexTree内部の2進セル1個ちょうどに
    /// 分解できる場合だけ受理する（例: `Interval::Second` の単一インデックス、または
    /// `Interval::Whole`）。Day/Hour/Minute単位の区間は2の冪秒ではないため単一セルに一致せず、
    /// エラーになる。[`RangeId`]（範囲）は常に成功する。この失敗しうる性質を名前で示すため
    /// `try_`接頭辞を付けている。
    fn try_with_temporal(self, temporal: TemporalId) -> Result<Self, Error>
    where
        Self: Sized;
}
