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
    /// 指定されたズームレベルの[RangeId]を出力する。
    ///
    /// [CoverSingleIds] の結果を単純に [RangeId] へ変換するラッパーではなく、
    /// 実装内部で [RangeId] の出力を活かす処理を持つこと。
    fn cover_range_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (crate::RangeId, Self::Value)>, crate::Error>;

    fn cover_range_ids(&self, z: u8) -> Result<impl Iterator<Item = crate::RangeId>, crate::Error> {
        Ok(self.cover_range_ids_with(z)?.map(|(id, _)| id))
    }
}

impl<T: CoverSingleIds, V: Clone> CoverSingleIds for (T, V) {
    type Value = V;
    fn cover_single_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (crate::SingleId, Self::Value)>, crate::Error> {
        let val = self.1.clone();
        Ok(self.0.cover_single_ids(z)?.map(move |id| (id, val.clone())))
    }
}

impl<T: CoverRangeIds, V: Clone> CoverRangeIds for (T, V) {
    type Value = V;
    fn cover_range_ids_with(
        &self,
        z: u8,
    ) -> Result<impl Iterator<Item = (crate::RangeId, Self::Value)>, crate::Error> {
        let val = self.1.clone();
        Ok(self.0.cover_range_ids(z)?.map(move |id| (id, val.clone())))
    }
}

pub trait CoverMapExt {
    type Shape;
    type Value;

    /// 図形を分解する前に、付随する値を事前に変換する
    fn map_value<U, F: FnOnce(Self::Value) -> U>(self, f: F) -> (Self::Shape, U);
}

impl<T, V> CoverMapExt for (T, V) {
    type Shape = T;
    type Value = V;
    fn map_value<U, F: FnOnce(Self::Value) -> U>(self, f: F) -> (Self::Shape, U) {
        (self.0, f(self.1))
    }
}
