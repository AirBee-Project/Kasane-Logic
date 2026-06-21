use crate::{BinaryOperator, ConflictPolicy, Error};

/// 和集合（A ∪ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合は[ConflictPolicy]に従って合成する。
/// - Aのみの場合はAが残る。
/// - Bのみの場合はBが残る。
///
/// # 性質
/// - 可換性：[ConflictPolicy::Min]か[ConflictPolicy::Max]の場合に可換
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
}
