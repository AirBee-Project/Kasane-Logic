use crate::{BinaryOperator, Error};

/// 対称差（A △ B）。どちらか一方にしか値がないセルだけを残す（重なりは捨てる）。
/// 左右が値を持ち寄るため、入力は同型 `V` に限る。
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

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        true
    }
}
