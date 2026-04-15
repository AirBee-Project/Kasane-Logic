use crate::{Error, RangeId, SingleId};

///SingleIdsがネイティブで実装されているもの
pub trait CoverSingleIds {
    /// あるズームレベルの[SingleId]を出力する。
    fn cover_single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error>;
}

///RangeIdsがネイティブで実装されているもの
pub trait CoverRangeIds {
    /// あるズームレベルの[RangeId]を出力する。
    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error>;
}

///FlexIdsがネイティブで実装されているもの
pub trait CoverFlexIds {}
