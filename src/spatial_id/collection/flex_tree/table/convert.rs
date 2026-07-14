use crate::{FlexId, SingleId, SpatialIdTable};

impl<V> SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    pub fn flex_ids(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.inner.single_ids()
    }
}
