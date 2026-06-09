use crate::{BinaryOperator, ConflictPolicy, Error};

/// 和集合（A ∪ B）。両方に値があるセルは [`ConflictPolicy`] で畳み込む。
pub struct Union;

impl<V: Ord + PartialEq + Clone> BinaryOperator<V, V> for Union {
    type CustomParameter = ConflictPolicy<V>;
    type ResultValue = V;

    fn both_some(a: &V, b: &V, policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(policy.resolve(Some(a.clone()), b.clone())))
    }

    fn a_only(a: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }

    fn is_commutative(policy: &Self::CustomParameter) -> bool {
        // 値方針が対称なときだけ可換。Overwrite/KeepExisting は左右で結果が変わる。
        // Fold はユーザ関数のため対称性を保証できず、保守的に非可換とみなす。
        matches!(policy, ConflictPolicy::Min | ConflictPolicy::Max)
    }
}
