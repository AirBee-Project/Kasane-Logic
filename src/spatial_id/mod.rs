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
    fn min_f(&self) -> i64;
    fn max_f(&self) -> i64;
    fn max_xy(&self) -> u64;

    //各インデックスの移動
    fn move_f(&mut self, by: i64) -> Result<(), Error>;
    fn move_x(&mut self, by: i64);
    fn move_y(&mut self, by: i64) -> Result<(), Error>;

    //各次元の長さを取得するメソット
    fn length_f(&self) -> f64;
    fn length_x(&self) -> f64;
    fn length_y(&self) -> f64;

    //中心点の座標を求める関数
    fn center(&self) -> Coordinate;

    //頂点をの座標を求める関数
    fn vertices(&self) -> [Coordinate; 8];

    //時間に関する設定をする関数
    fn t(&self) -> [u64; 2];
    fn set_t(&mut self, range: [u64; 2]);

    //時間を移動させる関数（自動的に実装）
    fn move_t(&mut self, by: i64) -> Result<(), Error> {
        let current = self.t();
        let mut next = [0u64; 2];

        for i in 0..2 {
            next[i] = if by >= 0 {
                current[i].checked_add(by as u64)
            } else {
                current[i].checked_sub(by.unsigned_abs())
            }
            .ok_or(Error::TOutOfRange {
                current: current[i],
                offset: by,
            })?;
        }

        self.set_t(next);
        Ok(())
    }
}

/// HyperRectはその時空間IDが各次元方向に連続で直方体（超直方体）であることを保証している
pub trait HyperRect: Clone {
    fn segmentation(&self) -> HyperRectSegments;
}

pub trait FlexIds {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId>;
}

impl<T: HyperRect> FlexIds for T {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId> {
        let HyperRectSegments { segments } = self.segmentation();

        // 0:F, 1:X, 2:Y, 3:T の順で分解
        let [f_vec, x_vec, y_vec, t_vec] = segments;

        f_vec.into_iter().flat_map(move |f_seg| {
            let x_vec = x_vec.clone();
            let y_vec = y_vec.clone();
            let t_vec = t_vec.clone();

            x_vec.into_iter().flat_map(move |x_seg| {
                let y_vec = y_vec.clone();
                let t_vec = t_vec.clone();
                let f_seg = f_seg.clone();

                y_vec.into_iter().flat_map(move |y_seg| {
                    let t_vec = t_vec.clone();
                    let f_seg = f_seg.clone();
                    let x_seg = x_seg.clone();

                    t_vec.into_iter().map(move |t_seg| {
                        FlexId::new(f_seg.clone(), x_seg.clone(), y_seg.clone(), t_seg)
                    })
                })
            })
        })
    }
}

/// RangeIDやSingleIDやFlexIDを最適分割したもの
pub struct HyperRectSegments {
    /// 0: F (高度), 1: X (東西), 2: Y (南北), 3: T (時間)
    pub segments: [Vec<Segment>; 4],
}
