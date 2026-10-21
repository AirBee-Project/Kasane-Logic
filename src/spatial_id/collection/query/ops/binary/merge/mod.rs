pub(crate) mod query;

#[cfg(test)]
mod test;

use crate::Error;
use crate::spatial_id::collection::query::ops::binary::run_binary;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{
    merge_policy::MergePolicy, query_iter::QueryIter, traits::BinaryOperator,
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
        &'a self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        let default = &self.default;
        run_binary(
            lhs,
            rhs,
            Some(move |a| P::resolve(a, default.clone())),
            Some(move |b| P::resolve(default.clone(), b)),
            Some(P::resolve),
        )
    }
}
