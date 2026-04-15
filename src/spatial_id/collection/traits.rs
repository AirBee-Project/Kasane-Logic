use crate::SpatialId;
use std::{
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor, RangeInclusive, Sub},
};

pub trait SpatialIdSet: Sized + Default + Eq + Clone + Debug
// --------------------------------------------------------
// [1] 両方消費パターン (Self OP Self)
// --------------------------------------------------------
+ BitOr<Output = Self>          // A | B (Union)
+ BitAnd<Output = Self>         // A & B (Intersection)
+ Sub<Output = Self>            // A - B (Difference)
+ BitXor<Output = Self>         // A ^ B (Symmetric Difference)
{
    ///新しい[SpatialIdSet]を作成する
    fn new() -> Self {
        Self::default()
    }

    ///[SpatialIds]を[SpatialIdSet]に挿入する
    fn insert<T: SpatialId>(&mut self, target: T);

    ///[SpatialIds]と重なる領域を取得する
    fn get<T: SpatialId>(&self, target: T) -> Self;

    ///[SpatialIds]と重なる領域を[SpatialIdSet]から削除し、取得する
    fn remove<T: SpatialId>(&mut self, target: T) -> Self;

    ///[SpatialIdSet]内にある単位空間の数を返す。集合の大体のサイズが分かる。
    fn size(&self) -> usize;

    /// [SpatialIdSet]の内部を空にする
    fn clear(&mut self);

    /// [SpatialIdSet]の内部が空かどうかを判定する
    fn is_empty(&self) -> bool;
}

///[SpatialId]に割り当てられた値を管理する
/// また、値側にも自動でインデックスを張り、高速なフィルターを提供する
pub trait SpatialIdTable<V>: Sized + Eq + Default + Clone + Debug
where
    V: Ord + Eq,
    for<'a> &'a Self::Set: BitOr<&'a Self::Set, Output = Self::Set>
        + BitAnd<&'a Self::Set, Output = Self::Set>
        + Sub<&'a Self::Set, Output = Self::Set>
        + BitXor<&'a Self::Set, Output = Self::Set>,
{
    type Set: SpatialIdSet;

    ///新しい[SpatialIdTable]を作成する
    fn new() -> Self;

    ///[SpatialIds]を[SpatialIdSet]に挿入する
    fn insert<T: SpatialId>(&mut self, target: T, value: V);

    ///[SpatialIds]と重なる領域を取得する
    fn get<T: SpatialId>(&self, target: T) -> Self;

    ///[SpatialIds]と重なる領域を[SpatialIdSet]から削除し、取得する
    fn remove<T: SpatialId>(&mut self, target: T) -> Self;

    ///等しい値の[SpatialId]を取り出して、[Some(Self::Set)]に格納して取り出す。
    /// 当該の値が存在しない場合は[None]を返す。
    fn find(&self, value: V) -> Option<Self::Set>;

    ///条件を満たす[SpatialId]を取り出して、[Some(Self::Set)]に格納して取り出す。
    /// 当該の値が存在しない場合は[None]を返す。
    fn filter(&self, range: RangeInclusive<V>) -> Option<Self::Set>;

    ///[SpatialIdSet]内にある単位空間の数を返す。集合の大体のサイズが分かる。
    fn size(&self) -> usize;

    /// [SpatialIdSet]の内部を空にする
    fn clear(&mut self);

    /// [SpatialIdSet]の内部が空かどうかを判定する
    fn is_empty(&self) -> bool;
}
