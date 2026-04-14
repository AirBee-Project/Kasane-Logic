
use crate::{Coordinate, FlexId, SingleId, TemporalId, error::Error};

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpatialId {
    //そのIDの各次元インデックス値の最大と最小を返す
    fn f_min(&self) -> i32;
    fn f_max(&self) -> i32;
    fn xy_max(&self) -> u32;
    fn xy_min(&self) -> u32 {
        0
    }

    //各インデックスの移動
    fn move_f(&mut self, by: i32) -> Result<(), Error>;
    fn move_x(&mut self, by: i32);
    fn move_y(&mut self, by: i32) -> Result<(), Error>;

    //各次元の長さを取得するメソット
    fn length_f_meters(&self) -> f64;
    fn length_x_meters(&self) -> f64;
    fn length_y_meters(&self) -> f64;

    //中心点の座標を求める関数
    fn spatial_center(&self) -> Coordinate;

    //頂点をの座標を求める関数
    fn spatial_vertices(&self) -> [Coordinate; 8];

    //時間が関連するもの
    fn temporal(&self) -> &TemporalId;
    fn temporal_mut(&mut self) -> &mut TemporalId;
}

///[SingleId]の集合であることを保証するTrait
pub trait IntoSingleIds {
    type IntoIter: Iterator<Item = SingleId>;
    fn into_single_ids(self) -> Self::IntoIter;
}
pub trait IterSingleIds {
    type Iter<'a>: Iterator<Item = SingleId> + 'a
    where
        Self: 'a;
    fn iter_single_ids(&self) -> Self::Iter<'_>;
}

///[FlexId]の集合であることを保証するTrait
pub trait IntoFlexIds {
    type IntoIter: Iterator<Item = FlexId>;
    fn into_flex_ids(self) -> Self::IntoIter;
}
pub trait IterFlexIds {
    type Iter<'a>: Iterator<Item = FlexId> + 'a
    where
        Self: 'a;
    fn iter_flex_ids(&self) -> Self::Iter<'_>;
}
