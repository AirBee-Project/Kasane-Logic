use super::MergePolicy;

/// 小さい方を採用するポリシー
pub struct Min;

impl<V: Ord> MergePolicy<V> for Min {
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a.min(b)
    }
}
