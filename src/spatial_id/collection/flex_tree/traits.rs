use alloc::boxed::Box;

use crate::spatial_id::collection::flex_tree::core::Summary;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::source::{Source, SourceIter};
use crate::{Error, FlexId, SpatialIdSet, SpatialIdTable};

impl Source for SpatialIdSet {
    type Value = ();

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        _token: &CancellationToken,
    ) -> Result<SourceIter<'a, ()>, Error> {
        Ok(Box::new(
            self.inner
                .get_overlapping(bounds.iter().copied())
                .map(|(id, ())| Ok((id, ()))),
        ))
    }
}

impl<V, S> Source for SpatialIdTable<V, S>
where
    V: SafeValue + 'static,
    S: Summary<V> + MaybeSendSync + 'static,
{
    type Value = V;

    fn read_flex_ids<'a>(
        &'a self,
        bounds: &'a [FlexId],
        _token: &CancellationToken,
    ) -> Result<SourceIter<'a, V>, Error> {
        Ok(Box::new(
            self.inner
                .get_overlapping(bounds.iter().copied())
                .map(|(id, value)| Ok((id, value.clone()))),
        ))
    }
}
