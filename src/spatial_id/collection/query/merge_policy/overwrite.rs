use super::MergePolicy;

/// 後から来た候補で既存の値を上書きする[MergePolicy]。
pub struct Overwrite;

impl<V> MergePolicy<V> for Overwrite {
    const IS_COMMUTATIVE: bool = false;
    const NAME: &'static str = "Overwrite";

    fn resolve(_a: V, b: V) -> V {
        b
    }
}
