use core::ops::Add as StdAdd;

use crate::{BinaryOperator, CellValue, Error};

/// 加算(A+B)を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合は値同士を足し合わせる。
/// - 片方にしか値がない場合は存在した値を維持する。（Noneを0として解釈する）
///
/// # 性質
/// - 可換性：可換
pub struct Add;

impl<V> BinaryOperator<V, V> for Add
where
    V: CellValue + StdAdd<Output = V>,
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
