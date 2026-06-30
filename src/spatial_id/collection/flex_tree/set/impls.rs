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
