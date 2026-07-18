use super::MergePolicy;

/// 大きい方を採用するポリシー
pub struct Max;

impl<V: Ord> MergePolicy<V> for Max {
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a.max(b)
    }
}
