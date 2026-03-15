use crate::SpatialId;
use std::{fmt::Debug, ops::RangeInclusive};

pub trait SpatialIdSet: Sized + Eq + Default + Clone + Debug
// --------------------------------------------------------
// [1] 両方消費パターン (Self OP Self)
// --------------------------------------------------------
// + BitOr<Output = Self>          // A | B (Union)
// + BitAnd<Output = Self>         // A & B (Intersection)
// + Sub<Output = Self>            // A - B (Difference)
// + BitOr<Output = Self>         // A ^ B (Symmetric Difference)

// --------------------------------------------------------
// [2] 片方消費パターン (Self OP &Self)
// --------------------------------------------------------
// + for<'a> BitOr<&'a Self, Output = Self>
// + for<'a> BitAnd<&'a Self, Output = Self>
// + for<'a> Sub<&'a Self, Output = Self>
// + for<'a> BitXor<&'a Self, Output = Self>

// --------------------------------------------------------
// [3] 破壊的代入パターン (Self OP= Self / &Self)
// --------------------------------------------------------
// + BitOrAssign<Self>
// + BitAndAssign<Self>
// + SubAssign<Self>
// + BitXorAssign<Self>
// + for<'a> BitOrAssign<&'a Self>
// + for<'a> BitAndAssign<&'a Self>
// + for<'a> SubAssign<&'a Self>
// + for<'a> BitXorAssign<&'a Self>
// where
//     // --------------------------------------------------------
//     // [4] 非破壊パターン (&Self OP &Self)
//     // --------------------------------------------------------
//     for<'a> &'a Self: BitOr<&'a Self, Output = Self>
//         + BitAnd<&'a Self, Output = Self>
//         + Sub<&'a Self, Output = Self>
//         + BitXor<&'a Self, Output = Self>,
{
    ///新しい[SpatialIdSet]を作成する
    fn new() -> Self {
        Self::default()
    }

    ///[SpatialId]を[SpatialIdSet]に挿入する
    fn insert<T: SpatialId>(&mut self, target: T);

    ///[SpatialId]と重なる領域を取得する
    fn get<T: SpatialId>(&self, target: T) -> Self;

    ///[SpatialId]と重なる領域を[SpatialIdSet]から削除し、取得する
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
pub trait SpatialIdTable<V>
where
    V: Ord + Eq,
{
    ///新しい[SpatialIdTable]を作成する
    fn new() -> Self;

    ///[SpatialId]を[SpatialIdSet]に挿入する
    fn insert<T: SpatialId>(&mut self, target: T, value: V);

    ///[SpatialId]と重なる領域を取得する
    fn get<T: SpatialId>(&self, target: T) -> Self;

    ///[SpatialId]と重なる領域を[SpatialIdSet]から削除し、取得する
    fn remove<T: SpatialId>(&mut self, target: T) -> Self;

    ///等しい値の[SpatialId]を取り出して、[Some(SpatialIdSet)]に格納して取り出す。
    /// 当該の値が存在しない場合は[None]を返す。
    fn find(&self, value: V) -> Option<impl SpatialIdSet>;

    ///条件を満たす[SpatialId]を取り出して、[Some(SpatialIdSet)]に格納して取り出す。
    /// 当該の値が存在しない場合は[None]を返す。
    fn filter(&self, range: RangeInclusive<V>) -> Option<impl SpatialIdSet>;

    ///[SpatialIdSet]内にある単位空間の数を返す。集合の大体のサイズが分かる。
    fn size(&self) -> usize;

    /// [SpatialIdSet]の内部を空にする
    fn clear(&mut self);

    /// [SpatialIdSet]の内部が空かどうかを判定する
    fn is_empty(&self) -> bool;
}

//現状の問題点
//Setを入力値として使うことができない
//SpatialIdsのようなTraitを実装しないといけない
//それは空間的範囲の意味を変化させないまま、

//相互変換について
//SingleIdやRangeIdをKeyとしたHashMapやBTreeMapへの変換に対応したほうが良いのではないか?
