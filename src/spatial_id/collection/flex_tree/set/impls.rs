use crate::{FlexId, SpatialIdSet};

impl FromIterator<FlexId> for SpatialIdSet {
    fn from_iter<T: IntoIterator<Item = FlexId>>(iter: T) -> Self {
        let mut set = SpatialIdSet::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl Extend<FlexId> for SpatialIdSet {
    fn extend<T: IntoIterator<Item = FlexId>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}
