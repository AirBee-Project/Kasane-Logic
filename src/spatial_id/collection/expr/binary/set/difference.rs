use crate::{BinaryOperator, Error};

/// 差集合（A - B）を行う二項演算。
///
/// # 計算内容
/// - Bの値がない場所にAの値を残す。
///
/// # 性質
/// - 可換性：非可換
pub struct Difference;

impl<A: Ord + PartialEq + Clone, B: Ord + PartialEq + Clone> BinaryOperator<A, B> for Difference {
    type CustomParameter = ();
    type ResultValue = A;

    fn both_some(_a: &A, _b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn a_only(a: &A, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(_b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        false
    }
}
