use super::MergePolicy;

/// 合計（足し算）を採用するポリシー
pub struct Sum;

impl<V: core::ops::Add<Output = V>> MergePolicy<V> for Sum {
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        a + b
    }
}
