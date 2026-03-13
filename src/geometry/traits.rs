use crate::{Coordinate, Error, RangeId, SingleId};

/// 現実空間の図形に対して共通で定義することができる性質
pub trait Geometry {
    fn center(&self) -> Coordinate;

    /// あるズームレベルの[SingleId]を出力する。
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error>;

    /// あるズームレベルの[RangeId]を出力する。
    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error>;

    /// 最小の個数の[SingleId]で出力する。
    ///
    /// 最小の個数を保証する。
    fn optimze_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        self.single_ids(z)
    }

    /// 最小の個数の[RangeId]で出力する。
    ///
    /// 最小の個数を保証する。
    fn optimze_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error> {
        self.range_ids(z)
    }
}
