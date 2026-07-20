use super::MergePolicy;
use core::ops::Sub;

/// 引き算（a - b）を採用するポリシー。
///
/// `b` が `a` より大きく結果を表現できない場合（符号なし整数のアンダーフロー等）は、
/// パニックやラップアラウンドを起こす代わりに `V::default()`（数値型では通常 0）にクランプします。
pub struct Difference;

impl<V: Sub<Output = V> + PartialOrd + Default> MergePolicy<V> for Difference {
    const IS_COMMUTATIVE: bool = false;

    fn resolve(a: V, b: V) -> V {
        if a >= b { a - b } else { V::default() }
    }
}
