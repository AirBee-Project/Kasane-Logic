use crate::{IterFlexIds, SpatialIdSet};

impl<S: IterFlexIds> FromIterator<S> for SpatialIdSet {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut set = SpatialIdSet::new();
        for items in iter {
            for item in items.iter_flex_ids() {
                set.insert(item);
            }
        }
        set
    }
}

impl<S: IterFlexIds> Extend<S> for SpatialIdSet {
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        for items in iter {
            for item in items.iter_flex_ids() {
                self.insert(item);
            }
        }
    }
}

impl IntoIterator for SpatialIdSet {
    type Item = crate::FlexId;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // SpatialIdSet の所有権を奪うイテレータ。現在は一旦 Vec に収集して返す。
        self.iter().collect::<alloc::vec::Vec<_>>().into_iter()
    }
}

impl<'a> IntoIterator for &'a SpatialIdSet {
    type Item = crate::FlexId;
    type IntoIter = alloc::boxed::Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        alloc::boxed::Box::new(self.iter())
    }
}
