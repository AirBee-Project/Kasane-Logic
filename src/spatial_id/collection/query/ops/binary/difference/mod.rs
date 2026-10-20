pub mod query;

use crate::Error;
use crate::spatial_id::collection::query::ops::binary::run_binary;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{query_iter::QueryIter, traits::BinaryOperator};

pub struct Difference<V> {
    _marker: core::marker::PhantomData<V>,
}

impl<V> Difference<V> {
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V> Default for Difference<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: SafeValue + 'static> BinaryOperator<V> for Difference<V> {
    fn run<'a>(
        &'a self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        run_binary(
            lhs,
            rhs,
            Some(core::convert::identity),
            None::<fn(V) -> V>,
            None::<fn(V, V) -> V>,
        )
    }
}
