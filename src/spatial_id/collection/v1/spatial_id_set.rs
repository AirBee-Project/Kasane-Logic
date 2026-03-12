use std::collections::BTreeSet;

use crate::{SingleId, SpatialIdSet};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq)]
pub struct SpatialIdSetV1 {
    inner: BTreeSet<SingleId>,
}

impl SpatialIdSet for SpatialIdSetV1 {
    fn insert<T: crate::SpatialId>(&mut self, target: T) {
        todo!()
    }

    fn get<T: crate::SpatialId>(&self, target: T) -> Self {
        todo!()
    }

    fn remove<T: crate::SpatialId>(&mut self, target: T) -> Self {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn fast_union(sets: impl IntoIterator<Item = Self>) -> Self {
        todo!()
    }

    fn fast_intersection(sets: impl IntoIterator<Item = Self>) -> Self {
        todo!()
    }
}
