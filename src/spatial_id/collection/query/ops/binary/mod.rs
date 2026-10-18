use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::traits::QueryIter;
use crate::{Error, SpatialIdTable};

pub mod difference;
pub mod intersection;
pub mod merge;

pub(crate) fn collect_both<V: SafeValue + 'static>(
    lhs: QueryIter<'_, V>,
    rhs: QueryIter<'_, V>,
) -> Result<(SpatialIdTable<V>, SpatialIdTable<V>), Error> {
    #[cfg(feature = "rayon")]
    let (a, b) = rayon::join(|| lhs.into_table(), || rhs.into_table());
    #[cfg(not(feature = "rayon"))]
    let (a, b) = (lhs.into_table(), rhs.into_table());
    Ok((a?, b?))
}
