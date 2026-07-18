use alloc::vec::Vec;

use super::merge_policy::MergePolicy;
use crate::{Error, FlexId, SpatialIdCollection};

/// 複数の `(FlexId, Value)` を `MergePolicy` に従ってコレクションに挿入します。
/// 空間的な重なりがある場合、元の領域をくり抜いて合成演算を適用し、残りの空間を補完します。
pub fn insert_with_policy<S, P>(target: &mut S, items: Vec<(FlexId, S::Value)>) -> Result<(), Error>
where
    S: SpatialIdCollection,
    P: MergePolicy<S::Value>,
{
    for (id, value) in items {
        let overlapping: Vec<_> = target.try_remove(&id)?.collect();
        let mut empty_spaces = alloc::vec![id.clone()];

        for (overlap_id, old_val) in overlapping {
            let merged_val = P::resolve(old_val, value.clone());
            target.try_insert(overlap_id.clone(), merged_val)?;

            let mut new_empty = Vec::new();
            for space in empty_spaces {
                new_empty.extend(space.difference(&overlap_id));
            }
            empty_spaces = new_empty;
        }

        for space in empty_spaces {
            target.try_insert(space, value.clone())?;
        }
    }
    Ok(())
}
