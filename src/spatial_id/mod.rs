use crate::{Coordinate, FlexId, Segment, error::Error};

pub(crate) mod collection;
pub mod constants;
pub(crate) mod range_id;
pub(crate) mod single_id;

//非公開のモジュール
pub(crate) mod flex_id;
pub(crate) mod helpers;
pub mod segment;

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
    fn length_f(&self) -> f64;
    fn length_x(&self) -> f64;
    fn length_y(&self) -> f64;

    //中心点の座標を求める関数
    fn center(&self) -> Coordinate;

    //頂点をの座標を求める関数
    fn vertices(&self) -> [Coordinate; 8];
}

/// 領域を構成するセグメントの集合を提供するトレイト
/// HyperRectはその空間IDが各次元方向に連続で直方体であることを保証している
pub trait HyperRect: Clone {
    fn segmentation(&self) -> HyperRectSegments;
}

/// FlexIdの集合として振る舞えるトレイト
/// Segmentationを実装している型には自動的に実装されます。
pub trait FlexIds {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId>;
}

impl<T: HyperRect> FlexIds for T {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId> {
        let HyperRectSegments { f, x, y } = self.segmentation();
        f.into_iter().flat_map(move |f_seg| {
            let x = x.clone();
            let y = y.clone();
            x.into_iter().flat_map(move |x_seg| {
                let y = y.clone();
                let f_seg = f_seg.clone();
                y.into_iter()
                    .map(move |y_seg| FlexId::new(f_seg.clone(), x_seg.clone(), y_seg.clone()))
            })
        })
    }
}

/// RangeIDやSingleIDやFlexIDを最適分割したもの
/// 必ず一続きの領域（直方体状の空間）を表す
pub struct HyperRectSegments {
    pub f: Vec<Segment>,
    pub x: Vec<Segment>,
    pub y: Vec<Segment>,
}
