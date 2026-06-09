use crate::{BinaryOperator, Error};

/// 差集合（A ∖ B）。`a_only` だけを残すため、B は重なり判定（presence）にのみ使われ、
/// その値型は問わない（`Set` でも別型の `Table` でもマスクとして使える）。結果は A の値を保つ。
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
