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
        // 件数を u16 で数えると 65535 件で飽和するので、`V` で数えて `V` の範囲まで数えられるようにする
        let (sum, count) = iter.fold((first, V::from(1u16)), |(sum, count), x| {
            (sum.saturating_add(x), count.saturating_add(V::from(1u16)))
        });
        Some(sum / count)
    }
}

#[cfg(test)]
mod test {
    use super::Average;
    use crate::spatial_id::collection::query::merge_policy::MergePolicy;

    #[test]
    fn resolve_many_counts_beyond_u16() {
        assert_eq!(
            <Average as MergePolicy<i32>>::resolve_many(core::iter::repeat_n(1, 100_000)),
            Some(1)
        );
        assert_eq!(
            <Average as MergePolicy<i32>>::resolve_many([1, 2, 3].into_iter()),
            Some(2)
        );
    }
}
