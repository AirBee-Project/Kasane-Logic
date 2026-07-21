use super::MergePolicy;
use core::ops::Sub;

/// 引き算を行う[MergePolicy]。
///
/// `b` が `a` より大きく結果を表現できない場合は、`V::default()`（数値型では通常 0）となります。
pub struct Difference;

impl<V: Sub<Output = V> + PartialOrd + Default> MergePolicy<V> for Difference {
    const IS_COMMUTATIVE: bool = false;
    const NAME: &'static str = "Difference";

    fn resolve(a: V, b: V) -> V {
        if a >= b { a - b } else { V::default() }
    }
}
