use crate::FlexId;
use crate::spatial_id::collection::query::send_sync::MaybeSendSync;

pub mod extrude;
pub mod falloff;
pub mod filter_values;
pub mod shift;
pub mod zoom_out;

/// 同じ [`FlexId`] の要素は入力の順を保つ。非可換の [`MergePolicy`](crate::spatial_id::collection::query::merge_policy::MergePolicy) の結果を実行ごとに変えないため。
pub(crate) fn sort_by_flex_id<T: MaybeSendSync>(items: &mut [(FlexId, T)]) {
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        items.par_sort_by_key(|a| a.0.field_key());
    }
    #[cfg(not(feature = "rayon"))]
    items.sort_by_key(|a| a.0.field_key());
}
