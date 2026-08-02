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

    /// 時間間隔 `{i}`（単位、秒数）を返す。
    ///
    /// [`SingleId`]/[`RangeId`]は設定された単位、[`FlexId`]はそのセルの秒幅
    /// （`2^(35 - t_zoomlevel)`）を返す。時間を設定していなければ
    /// [`Interval::WHOLE`]。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, SingleId, SpatialId};
    /// let id = SingleId::new(12, 0, 3638, 1614).unwrap().with_time(1800, 809712).unwrap();
    /// assert_eq!(id.interval().seconds(), 1800);
    ///
    /// let plain = SingleId::new(12, 0, 3638, 1614).unwrap();
    /// assert_eq!(plain.interval(), Interval::WHOLE);
    /// # }
    /// ```
    fn interval(&self) -> Interval;

    /// 占有する絶対秒区間 `[start, end)` を返す（1970-01-01 00:00 UTC 起点）。
    ///
    /// 実装型ごとに時間の形（単一 / 範囲 / 木のセル）は違うが、秒区間へ落とせば共通に
    /// 扱える。空間側で `f()` の戻り値が `i32` と `[i32; 2]` に分かれるため、Traitには
    /// `f_min()` / `f_max()` だけを載せているのと同じ方針である。
    ///
    /// 時間の**付与**はこのTraitには含めない。3型とも `with_time` という同じ名前を持つが、
    /// 受け取る型がその型の「母語」に従って違うためである。
    ///
    /// | 型 | `with_time` の引数 |
    /// |---|---|
    /// | [`SingleId`] | 秒数（[`Interval`]）＋単一の `{t}` |
    /// | [`RangeId`] | 秒数（[`Interval`]）＋ `{t}` の範囲 |
    /// | [`FlexId`] | **[`TZoomLevel`](crate::TZoomLevel)** ＋セルのインデックス |
    ///
    /// [`FlexId`]だけズームを取るのは、木のノードアドレスとして `2^(35 - zoom)` 秒の
    /// 2進セル1個しか保持できないからである。型が違うので取り違えはコンパイルエラーになる。
    /// 実時間で指定したいだけなら、3型に共通の `with_time_span` / `with_time_at` を使う。
    ///
    /// 任意秒数の間隔（`1800` など）を[`FlexId`]の集合として扱いたい場合は、
    /// [`SingleId`]/[`RangeId`]に付けてから[`IntoIterator`]で展開する
    /// （必要な数のセルへ自動的に分解される）。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::{Interval, RangeId, SpatialId};
    /// let id = RangeId::new(4, 0, 0, 0).unwrap().with_time(Interval::HOUR, [1, 2]).unwrap();
    /// assert_eq!(id.seconds_range(), (3600, 10800));
    /// # }
    /// ```
    fn seconds_range(&self) -> (u64, u64);

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
