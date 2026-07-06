use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{FlexId, IterFlexIds, IterSingleIds, SingleId, SpatialIdTable};

impl<V> IterFlexIds for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    type Iter<'a>
        = Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        Box::new(self.iter().map(|(flex_id, _)| flex_id))
    }
}

impl<V> IterSingleIds for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    type Iter<'a>
        = alloc::vec::IntoIter<SingleId>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        let ids: Vec<SingleId> = self.flat_single_ids().map(|(single, _)| single).collect();
        ids.into_iter()
    }
}
