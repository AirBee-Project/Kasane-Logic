use super::MergePolicy;
use super::saturating_add::Add;
use core::ops::Div;

/// 大きな値を空間に残す[MergePolicy]。
pub struct Average;

impl<V> MergePolicy<V> for Average
where
    V: Add + Div<V, Output = V> + From<u16>,
{
    const IS_COMMUTATIVE: bool = false;

    fn resolve(a: V, b: V) -> V {
        a.saturating_add(b) / V::from(2u16)
    }

    fn resolve_many(mut iter: impl Iterator<Item = V>) -> Option<V> {
        let first = iter.next()?;
        let mut count = 1u16;
        let sum = iter.fold(first, |acc, x| {
            count = count.saturating_add(1);
            acc.saturating_add(x)
        });
        Some(sum / V::from(count))
    }
}
