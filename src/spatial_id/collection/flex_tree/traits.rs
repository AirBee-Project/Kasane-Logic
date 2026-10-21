use crate::spatial_id::collection::flex_tree::core::Summary;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{query_iter::QueryIter, traits::Source};
use crate::{Error, RangeId, SpatialIdSet, SpatialIdMap};

impl Source for SpatialIdSet {
    type Value = ();

    fn read_flex_ids<'a>(
        &'a self,
        targets: &[RangeId],
        _token: &CancellationToken,
    ) -> Result<QueryIter<'a, ()>, Error> {
        Ok(QueryIter::from_ordered(
            self.inner
                .get_overlapping(targets.iter().cloned())
                .map(|(id, ())| Ok((id, ()))),
        ))
    }
}

impl<V, S> Source for SpatialIdMap<V, S>
where
    V: SafeValue + 'static,
    S: Summary<V> + MaybeSendSync + 'static,
{
    type Value = V;

    fn read_flex_ids<'a>(
        &'a self,
        targets: &[RangeId],
        _token: &CancellationToken,
    ) -> Result<QueryIter<'a, V>, Error> {
        Ok(QueryIter::from_ordered(
            self.inner
                .get_overlapping(targets.iter().cloned())
                .map(|(id, value)| Ok((id, value.clone()))),
        ))
    }
}
