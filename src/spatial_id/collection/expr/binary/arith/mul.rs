use core::ops::Mul as StdMul;

use crate::{BinaryOperator, CellValue, Error};
/// 乗算(A×B)を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合は値同士を掛け合わせる。
/// - 片方にのみ値がある場合は0となる。(Noneを0として解釈する)
///
/// # 性質
/// - 可換性：可換
pub struct Mul;

impl<V> BinaryOperator<V, V> for Mul
where
    V: CellValue + StdMul<Output = V>,
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
