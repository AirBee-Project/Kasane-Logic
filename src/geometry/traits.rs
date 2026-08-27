pub trait CoverSingleIds {
    type Value;
    /// 対象の図形が覆う `SingleId` の集合を返す。
    fn cover_single_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (crate::SingleId, Self::Value)>, crate::Error>;

    fn cover_single_ids(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = crate::SingleId>, crate::Error> {
        Ok(self.cover_single_ids_with(z)?.map(|(id, _)| id))
    }
}

pub trait CoverRangeIds {
    type Value;
    /// 指定されたズームレベルの[crate::RangeId]を出力する。
    ///
    /// [CoverSingleIds] の結果を単純に [crate::RangeId] へ変換するラッパーではなく、
    /// 実装内部で [crate::RangeId] の出力を活かす処理を持つこと。
    fn cover_range_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (crate::RangeId, Self::Value)>, crate::Error>;

    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::RangeId>, crate::Error> {
        Ok(self.cover_range_ids_with(z)?.map(|(id, _)| id))
    }
}
