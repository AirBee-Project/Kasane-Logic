//! 時空間ID操作のトレイトとその実装
//!
//! このモジュールは、時空間IDの基本的な操作を定義する [`SpaceId`] トレイトと、
//! その実装（単一ID、範囲ID）を提供します。

use crate::error::Error;

pub mod constants;
pub mod range;
pub mod single;
pub mod segment;

pub trait SpaceId {
    //そのIDの各次元の最大と最小を返す
    fn min_f(&self) -> i64;
    fn max_f(&self) -> i64;
    fn max_xy(&self) -> u64;

    //垂直方向に動かす
    fn move_up(&mut self, by: i64) -> Result<(), Error>;
    fn move_down(&mut self, by: i64) -> Result<(), Error>;

    //水平方向に動かす
    fn move_north(&mut self, by: u64);
    fn move_south(&mut self, by: u64);
    fn move_east(&mut self, by: u64);
    fn move_west(&mut self, by: u64);
}

// Legacy re-export for compatibility
pub use SpaceId as SpaceID;
