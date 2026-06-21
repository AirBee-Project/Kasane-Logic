use crate::{BinaryOperator, Error};

/// 対称差（A △ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合はNoneにする。
/// - Aのみの場合はAが残る。
/// - Bのみの場合はBが残る。
///
/// # 性質
/// - 可換性：可換
pub struct SymmetricDifference;

impl<V: Ord + PartialEq + Clone> BinaryOperator<V, V> for SymmetricDifference {
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(_a: &V, _b: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn a_only(a: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }
}
