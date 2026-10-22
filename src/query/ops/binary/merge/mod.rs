pub(crate) mod query;

#[cfg(test)]
mod test;

use crate::Error;
use crate::query::SafeValue;
use crate::query::ops::binary::run_binary;
use crate::query::{merge_policy::MergePolicy, query_iter::QueryIter, traits::BinaryOperator};

/// `MergePolicy` で重ね合わせる二項演算子。
pub struct Merge<V, P> {
    _marker: core::marker::PhantomData<fn() -> (V, P)>,
}

impl<V, P> Default for Merge<V, P> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V, P> Merge<V, P> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<V: SafeValue + 'static, P> BinaryOperator<V> for Merge<V, P>
where
    P: MergePolicy<V>,
{
    fn run<'a>(
        &'a self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        run_binary(lhs, rhs, Some(|a| a), Some(|b| b), Some(P::resolve))
    }
}
