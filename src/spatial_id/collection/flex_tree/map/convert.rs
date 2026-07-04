use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SingleId, SpatialIdMap,
};

impl<V> IntoFlexIds for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type IntoIter = alloc::vec::IntoIter<FlexId>;

    fn into_flex_ids(self) -> Self::IntoIter {
        let ids: Vec<FlexId> = self.iter().map(|(flex_id, _)| flex_id).collect();
        ids.into_iter()
    }
}

impl<V> IterFlexIds for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type Iter<'a>
        = Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        Box::new(self.iter().map(|(flex_id, _)| flex_id))
    }
}

impl<V> IntoSingleIds for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    type IntoIter = alloc::vec::IntoIter<SingleId>;

    fn into_single_ids(self) -> Self::IntoIter {
        let ids: Vec<SingleId> = self.flat_single_ids().map(|(single, _)| single).collect();
        ids.into_iter()
    }
}

impl<V> IterSingleIds for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
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
