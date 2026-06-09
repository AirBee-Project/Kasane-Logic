use core::ops::Add as StdAdd;

use crate::{BinaryOperator, Error};

/// 加算（A + B）。両方に値があるセルは値同士を足し合わせ、片側にしか値がないセルは
/// その値をそのまま残す（欠落側を加法の単位元 `0` とみなす和）。
///
/// 値型 `V` は加算 [`core::ops::Add`] を実装している必要がある。加算が可換であることを前提に
/// [`is_commutative`](BinaryOperator::is_commutative) は常に `true` を返す。
pub struct Add;

impl<V> BinaryOperator<V, V> for Add
where
    V: Ord + PartialEq + Clone + StdAdd<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() + b.clone()))
    }

    fn a_only(a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        true
    }
}
