use crate::{BinaryOperator, Error};

/// マスク（A を B の存在範囲で切り取る）。構造は積集合だが、重なりセルには常に A の値を残す。
/// B は presence にのみ使われ値型を問わないため、`Set` や別型 `Table` でマスクできる。
pub struct Mask;

impl<A: Ord + PartialEq + Clone, B: Ord + PartialEq + Clone> BinaryOperator<A, B> for Mask {
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
