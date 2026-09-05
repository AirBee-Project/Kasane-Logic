pub trait CoverSingleIds {
    /// 対象の図形が覆う `SingleId` の集合を返す。
    fn cover_single_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (crate::SingleId, V)>, crate::Error>
    where
        V: Clone + 'static;

    fn cover_single_ids(
        &self,
        z: impl Into<u8>,
    ) -> Result<impl Iterator<Item = crate::SingleId>, crate::Error> {
        Ok(self.cover_single_ids_with(z, ())?.map(|(id, _)| id))
    }
}

pub trait CoverRangeIds {
    /// 指定されたズームレベルの[crate::RangeId]を出力する。
    ///
    /// [CoverSingleIds] の結果を単純に [crate::RangeId] へ変換するラッパーではなく、
    /// 実装内部で [crate::RangeId] の出力を活かす処理を持つこと。
    fn cover_range_ids_with<V>(
        &self,
        z: impl Into<u8>,
        value: V,
    ) -> Result<impl Iterator<Item = (crate::RangeId, V)>, crate::Error>
    where
        V: Clone + 'static;

    fn cover_range_ids(
        &self,
        z: impl Into<u8>,
    ) -> Result<impl Iterator<Item = crate::RangeId>, crate::Error> {
        Ok(self.cover_range_ids_with(z, ())?.map(|(id, _)| id))
    }
}
