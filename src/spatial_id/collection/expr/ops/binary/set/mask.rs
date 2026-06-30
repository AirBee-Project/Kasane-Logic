use crate::{BinaryOperator, CellValue, Error};

/// マスク（AをBの存在範囲で切り取る）二項演算。
///
/// # 計算内容
/// - Bに値が存在する部分をNoneにしたAを返す。
///
/// # 性質
/// - 可換性：非可換
pub struct Mask;

impl<A: CellValue, B: CellValue> BinaryOperator<A, B> for Mask {
    type CustomParameter = ();
    type ResultValue = A;

    fn both_some(a: &A, _b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(Some(a.clone()))
    }

    fn a_only(_a: &A, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn b_only(_b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        false
    }
}
