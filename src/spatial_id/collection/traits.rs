use crate::{SpatialId, SpatialIds};
use std::{
    fmt::Debug,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, RangeInclusive, Sub,
        SubAssign,
    },
};

pub trait SpatialIdSet: Sized + Default + Eq + Clone + Debug + SpatialIds
// --------------------------------------------------------
// [1] 両方消費パターン (Self OP Self)
// --------------------------------------------------------
+ BitOr<Output = Self>          // A | B (Union)
+ BitAnd<Output = Self>         // A & B (Intersection)
+ Sub<Output = Self>            // A - B (Difference)
+ BitXor<Output = Self>         // A ^ B (Symmetric Difference)

// --------------------------------------------------------
// [2] 片方消費パターン (Self OP &Self)
// --------------------------------------------------------
+ for<'a> BitOr<&'a Self, Output = Self>
+ for<'a> BitAnd<&'a Self, Output = Self>
+ for<'a> Sub<&'a Self, Output = Self>
+ for<'a> BitXor<&'a Self, Output = Self>

// --------------------------------------------------------
// [3] 破壊的代入パターン (Self OP= Self / &Self)
// --------------------------------------------------------
+ BitOrAssign<Self>
+ BitAndAssign<Self>
+ SubAssign<Self>
+ BitXorAssign<Self>
+ for<'a> BitOrAssign<&'a Self>
+ for<'a> BitAndAssign<&'a Self>
+ for<'a> SubAssign<&'a Self>
+ for<'a> BitXorAssign<&'a Self>
where
//     // --------------------------------------------------------
//     // [4] 非破壊パターン (&Self OP &Self)
//     // --------------------------------------------------------
    for<'a> &'a Self: BitOr<&'a Self, Output = Self>
        + BitAnd<&'a Self, Output = Self>
        + Sub<&'a Self, Output = Self>
        + BitXor<&'a Self, Output = Self>,
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
pub trait SpatialIdTable<V>: Sized + Eq + Default + Clone + Debug + SpatialIds
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
