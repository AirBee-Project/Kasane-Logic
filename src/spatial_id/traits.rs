use crate::{Coordinate, SingleId, error::Error, spatial_id::temporal_id::TemporalId};

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpatialId {
    //そのIDの各次元の最大と最小を返す
    fn min_f(&self) -> i32;
    fn max_f(&self) -> i32;
    fn max_xy(&self) -> u32;

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

    //SingleIdとして書き出すもの
    fn single_ids(&self) -> impl Iterator<Item = SingleId>;
    fn optimize_single_ids(&self) -> impl Iterator<Item = SingleId>;
}
