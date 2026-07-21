use super::MergePolicy;
use super::saturating_add::Add;
use core::ops::Div;

/// 大きな値を空間に残す[MergePolicy]。
pub struct Average;

impl<V> MergePolicy<V> for Average
where
    V: Add + Div<V, Output = V> + From<u16> + Send + Sync + 'static,
{
    const IS_COMMUTATIVE: bool = true;
    const NAME: &'static str = "Average";

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b) / V::from(2u16)
    }

    fn resolve_many(mut iter: impl ExactSizeIterator<Item = V>) -> Option<V> {
        let count = iter.len();
        let first = iter.next()?;
        let sum = iter.fold(first, Add::saturating_add);
        let count_v = V::from(count as u16);
        Some(sum / count_v)
    }
}
