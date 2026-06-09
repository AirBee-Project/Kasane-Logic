use core::ops::Sub as StdSub;

use crate::{BinaryOperator, Error};

/// 減算（A - B）。両方に値があるセルは差 `a - b` を取り、`a` だけのセルは `a` をそのまま残す。
/// `b` だけのセルは結果に出さない（A の定義域内に結果をとどめる）。
///
/// 値型 `V` は減算 [`core::ops::Sub`] を実装している必要がある。`b` のみのセルを捨てるため
/// 符号反転（`Neg`）は不要で、`u8` などの符号なし型にも使える。左右で結果が変わるため非可換。
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

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        false
    }
}
