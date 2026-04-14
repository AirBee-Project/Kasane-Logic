use std::borrow::Borrow;

use crate::{Coordinate, FlexId, SingleId, error::Error};

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
    #[cfg(feature = "temporal")]
    fn temporal(&self) -> &TemporalId;
    #[cfg(feature = "temporal")]
    fn temporal_mut(&mut self) -> &mut TemporalId;
}

///[SingleId]の集合であることを保証するTrait
pub trait SingleIds {
    type Item<'a>: Borrow<SingleId> + 'a
    where
        Self: 'a;

    fn single_ids(&self) -> impl Iterator<Item = Self::Item<'_>>;
}

///[FlexId]の集合であることを保証するTrait
pub trait FlexIds {
    type Item<'a>: Borrow<FlexId> + 'a
    where
        Self: 'a;

    fn flex_ids(&self) -> impl Iterator<Item = Self::Item<'_>>;
}
