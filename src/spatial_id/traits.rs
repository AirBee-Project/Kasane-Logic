use crate::{Coordinate, FlexId, SingleId, TemporalId, error::Error};

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpatialId {
    /// F 方向の最小インデックスを返す。
    fn f_min(&self) -> i32;

    /// F 方向の最大インデックスを返す。
    fn f_max(&self) -> i32;

    /// X 方向の最大インデックスを返す。
    fn x_max(&self) -> u32;

    /// X 方向の最小インデックスを返す。
    fn x_min(&self) -> u32 {
        0
    }

    /// Y 方向の最大インデックスを返す。
    fn y_max(&self) -> u32;

    /// Y 方向の最小インデックスを返す。
    fn y_min(&self) -> u32 {
        0
    }

    /// F 方向に指定量だけ移動する。
    fn move_f(&mut self, by: i32) -> Result<(), Error>;

    /// X 方向に指定量だけ移動する。
    fn move_x(&mut self, by: i32);

    /// Y 方向に指定量だけ移動する。
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
