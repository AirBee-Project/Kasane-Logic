use super::MergePolicy;

/// 小さな値を空間に残す[MergePolicy]
pub struct Min;

impl<V: Ord> MergePolicy<V> for Min {
    const IS_COMMUTATIVE: bool = true;
    const NAME: &'static str = "Min";

    fn resolve(a: V, b: V) -> V {
        a.min(b)
    }
}
