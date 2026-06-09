use crate::{BinaryOperator, ConflictPolicy, Error};

/// 積集合（A ∩ B）。両方に値があるセルだけを残し、値は [`ConflictPolicy`] で畳み込む。
pub struct Intersection;

impl<V: Ord + PartialEq + Clone> BinaryOperator<V, V> for Intersection {
    type CustomParameter = ConflictPolicy<V>;
    type ResultValue = V;

    fn both_some(a: &V, b: &V, policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(policy.resolve(Some(a.clone()), b.clone())))
    }

    fn a_only(_a: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn b_only(_b: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn is_commutative(policy: &Self::CustomParameter) -> bool {
        matches!(policy, ConflictPolicy::Min | ConflictPolicy::Max)
    }
}
