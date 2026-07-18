use super::MergePolicy;

/// 既存の値を保持し、後から来た候補を捨てるポリシー
pub struct KeepExisting;

impl<V> MergePolicy<V> for KeepExisting {
    const IS_COMMUTATIVE: bool = false;

    fn resolve(a: V, _b: V) -> V {
        a
    }
}
