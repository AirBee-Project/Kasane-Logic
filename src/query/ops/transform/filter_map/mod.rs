pub(crate) mod query;

#[cfg(test)]
mod test;

use crate::query::SafeValue;
use crate::query::cancellation::CancellationToken;
use crate::query::query_iter::QueryIter;
use crate::query::{Query, traits::Source};
use crate::{Error, FlexId, RangeId};

/// `inner` の各 `(FlexId, V)` を `f` で写し、[`None`] を返した要素を取り除く。
pub struct FilterMap<V: SafeValue + 'static, U, F> {
    inner: Query<V>,
    f: F,
    _marker: core::marker::PhantomData<fn() -> U>,
}

impl<V, U, F> Source for FilterMap<V, U, F>
where
    V: SafeValue + 'static,
    U: SafeValue + 'static,
    F: Fn(FlexId, V) -> Option<U> + Send + Sync + 'static,
{
    type Value = U;

    fn read_flex_ids<'a>(
        &'a self,
        targets: &[RangeId],
        token: &CancellationToken,
    ) -> Result<QueryIter<'a, U>, Error> {
        // `Query::validate` は入力源の中まで辿らないので、内側のクエリはここで検証する
        self.inner.validate()?;
        let inner = self.inner.stream_within(targets.to_vec(), token)?;
        Ok(inner.filter_map_ordered(&self.f))
    }
}
