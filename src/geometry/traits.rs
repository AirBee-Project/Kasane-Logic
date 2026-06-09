use crate::{Error, RangeId, SingleId};

pub trait CoverSingleIds {
    /// 指定されたズームレベルの[SingleId]を出力する。
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error>;
}

pub trait CoverRangeIds {
    /// 指定されたズームレベルの[RangeId]を出力する。
    ///
    /// [CoverSingleIds] の結果を単純に [RangeId] へ変換するラッパーではなく、
    /// 実装内部で [RangeId] の出力を活かす処理を持つこと。
    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error>;
}
