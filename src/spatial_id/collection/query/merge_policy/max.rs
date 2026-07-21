use super::MergePolicy;

/// 大きな値を空間に残す[MergePolicy]。
pub struct Max;

impl<V: Ord> MergePolicy<V> for Max {
    const IS_COMMUTATIVE: bool = true;
    const NAME: &'static str = "Max";

    fn resolve(a: V, b: V) -> V {
        a.max(b)
    }
}
