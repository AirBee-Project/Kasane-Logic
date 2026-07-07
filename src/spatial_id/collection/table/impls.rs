use crate::{FlexId, SpatialIdTable};

impl<V> FromIterator<(FlexId, V)> for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        let mut table = SpatialIdTable::new();
        for (id, val) in iter {
            table.insert(id, val);
        }
        table
    }
}

impl<V> Extend<(FlexId, V)> for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        for (id, val) in iter {
            self.insert(id, val);
        }
    }
}

impl<V> IntoIterator for SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    type Item = (FlexId, V);
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SpatialIdTable の所有権を奪うイテレータ。一旦 Vec に収集して返す。
        let vec: alloc::vec::Vec<_> = self.iter().map(|(id, v)| (id, v.clone())).collect();
        vec.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a SpatialIdTable<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue + Ord,
{
    type Item = (FlexId, &'a V);
    type IntoIter = alloc::boxed::Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        alloc::boxed::Box::new(self.iter())
    }
}
