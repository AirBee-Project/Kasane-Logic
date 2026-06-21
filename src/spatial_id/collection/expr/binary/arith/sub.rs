use core::ops::Sub as StdSub;

use crate::{BinaryOperator, Error};

/// 減算（A-B）を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合はA-Bを行う。
/// - Aにしか値がない場合は維持する。（BのNoneを0として解釈する）
/// - Bにしか値がない場合はNoneを出力する。（Aが存在しないため計算不能）
///
/// # 性質
/// - 可換性：非可換
pub struct Sub;

impl<V> BinaryOperator<V, V> for Sub
where
    V: Ord + PartialEq + Clone + StdSub<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() - b.clone()))
    }

    fn a_only(a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(_b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }
}
