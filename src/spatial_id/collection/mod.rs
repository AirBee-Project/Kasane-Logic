use crate::SpatialId;
use std::fmt::Debug;

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

    ///複数の[SpatialIdSet]のUnionを高速に取る
    fn fast_union(sets: impl IntoIterator<Item = Self>) -> Self;

    ///複数の[SpatialIdSet]のIntersectionを高速に取る
    fn fast_intersection(sets: impl IntoIterator<Item = Self>) -> Self;
}
