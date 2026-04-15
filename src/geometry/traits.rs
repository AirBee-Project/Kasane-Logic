use crate::{Error, RangeId, SingleId};

///SingleIdsがネイティブで実装されているもの
pub trait ToSingleIds {
    /// あるズームレベルの[SingleId]を出力する。
    fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error>;
}

///RangeIdsがネイティブで実装されているもの
pub trait ToRangeIds {
    /// あるズームレベルの[RangeId]を出力する。
    fn range_ids(&self, z: u8) -> Result<impl Iterator<Item = RangeId>, Error>;
}

///FlexIdsがネイティブで実装されているもの
pub trait ToFlexIds {}
