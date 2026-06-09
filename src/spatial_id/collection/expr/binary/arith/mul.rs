use core::ops::Mul as StdMul;

use crate::{BinaryOperator, Error};

/// 乗算（A × B）。両方に値があるセルだけ積 `a * b` を残し、片側にしか値がないセルは結果に
/// 出さない（欠落側を零元 `0` とみなす積）。重なる領域だけを取り出す積集合的な振る舞いで、
/// マスクや重み付けに使える。
///
/// 値型 `V` は乗算 [`core::ops::Mul`] を実装している必要がある。乗算が可換であることを前提に
/// [`is_commutative`](BinaryOperator::is_commutative) は常に `true` を返す。
pub struct Mul;

impl<V> BinaryOperator<V, V> for Mul
where
    V: Ord + PartialEq + Clone + StdMul<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() * b.clone()))
    }

    fn a_only(_a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn b_only(_b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        true
    }
}
