use crate::{
    Coordinate, FlexId, RangeId, Segment, SingleId, error::Error,
    spatial_id::temporal_id::TemporalId,
};

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpatialId {
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

pub struct Block {
    f: Vec<Segment<8>>,
    x: Vec<Segment<8>>,
    y: Vec<Segment<8>>,
}

pub trait SpatialIds {
    type SingleItem<'a>
    where
        Self: 'a;
    type RangeItem<'a>
    where
        Self: 'a;
    type FlexItem<'a>
    where
        Self: 'a;

    fn single_ids(&self) -> impl Iterator<Item = Self::SingleItem<'_>>;
    fn range_ids(&self) -> impl Iterator<Item = Self::RangeItem<'_>>;
    fn flex_ids(&self) -> impl Iterator<Item = Self::FlexItem<'_>>;
    fn block(&self) -> Option<Block>;
}
