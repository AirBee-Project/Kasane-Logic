use crate::{BinaryOperator, ConflictPolicy, Error};

/// 積集合（A ∩ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合は[ConflictPolicy]に従って合成する。
/// - どちらかが存在しない場合はNoneとなる。
///
/// # 性質
/// - 可換性：[ConflictPolicy::Min]か[ConflictPolicy::Max]の場合に可換
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
}
