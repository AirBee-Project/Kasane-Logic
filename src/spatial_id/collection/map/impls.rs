use crate::{FlexId, SpatialIdMap};

impl<V> FromIterator<(FlexId, V)> for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        let mut map = SpatialIdMap::new();
        for (id, val) in iter {
            map.insert(id, val);
        }
        map
    }
}

impl<V> Extend<(FlexId, V)> for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        for (id, val) in iter {
            self.insert(id, val);
        }
    }
}

impl<V> IntoIterator for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    type Item = (FlexId, V);
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SpatialIdMap の所有権を奪うイテレータ。一旦 Vec に収集して返す。
        self.iter()
            .map(|(id, v)| (id, v.clone()))
            .collect::<alloc::vec::Vec<_>>()
            .into_iter()
    }
}

impl<'a, V> IntoIterator for &'a SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    type Item = (FlexId, &'a V);
    type IntoIter = alloc::boxed::Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        alloc::boxed::Box::new(self.iter())
    }
}
