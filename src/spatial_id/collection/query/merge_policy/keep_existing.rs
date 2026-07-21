use super::MergePolicy;

/// 既存の値を保持し、後から来た値を捨てる[MergePolicy]。
pub struct KeepExisting;

impl<V> MergePolicy<V> for KeepExisting {
    const IS_COMMUTATIVE: bool = false;
    const NAME: &'static str = "KeepExisting";

    fn resolve(a: V, _b: V) -> V {
        a
    }
}
