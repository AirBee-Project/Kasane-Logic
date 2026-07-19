use super::MergePolicy;
use core::iter::Sum;
use core::ops::{Add, Div};

/// 平均（Average）を採用するポリシー。
///
/// `ZoomOut` などの一括集約操作では `resolve_many` が呼ばれ、真の平均（Sum / Count）が計算されます。
/// 逐次挿入時（`resolve`）では `(a + b) / 2` の移動平均となることに注意してください。
pub struct Average;

impl<V> MergePolicy<V> for Average
where
    V: Sum + Div<V, Output = V> + From<u16> + Add<Output = V> + Send + Sync + 'static,
{
    const IS_COMMUTATIVE: bool = true;

    fn resolve(a: V, b: V) -> V {
        (a + b) / V::from(2u16)
    }

    fn resolve_many(iter: impl ExactSizeIterator<Item = V>) -> Option<V> {
        let count = iter.len();
        if count == 0 {
            return None;
        }
        let sum: V = iter.sum();
        let count_v = V::from(count as u16);
        Some(sum / count_v)
    }
}
