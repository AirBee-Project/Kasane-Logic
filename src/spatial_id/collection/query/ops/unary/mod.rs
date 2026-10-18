use crate::FlexId;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;

pub mod extrude;
pub mod falloff;
pub mod filter_values;
pub mod shift;
pub mod zoom_out;

pub(crate) fn sort_by_flex_id<T: MaybeSendSync>(items: &mut [(FlexId, T)]) {
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        items.par_sort_unstable_by_key(|a| a.0);
    }
    #[cfg(not(feature = "rayon"))]
    items.sort_unstable_by_key(|a| a.0);
}
