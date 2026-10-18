use crate::Error;
use crate::spatial_id::collection::query::ops::binary::collect_both;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{
    merge_policy::MergePolicy,
    traits::{BinaryOperator, QueryIter},
};

/// `MergePolicy` で重ね合わせる二項演算子。
pub struct Merge<V, P> {
    default: V,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<V, P> Merge<V, P> {
    pub fn new(default: V) -> Self {
        Self {
            default,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V: SafeValue + 'static, P> BinaryOperator<V> for Merge<V, P>
where
    P: MergePolicy<V>,
{
    /// 両側に値がある場所は `resolve(a, b)`、片側だけの場所は欠けた側を `default` として解決する。
    fn run<'a>(
        &self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        let (mut target_a, target_b) = collect_both(lhs, rhs)?;
        target_a
            .inner
            .merge_with_default(&target_b.inner, &self.default, |x, y| {
                P::resolve(x.clone(), y.clone())
            });
        Ok(target_a.into())
    }
}
