use crate::{FlexTreeCore, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialIdSet};

impl IntoFlexIds for SpatialIdSet {
    type IntoIter = <FlexTreeCore<()> as IntoFlexIds>::IntoIter;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.inner.into_flex_ids()
    }
}

impl IterFlexIds for SpatialIdSet {
    type Iter<'a>
        = <FlexTreeCore<()> as IterFlexIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_flex_ids()
    }
}

impl IntoSingleIds for SpatialIdSet {
    type IntoIter = <FlexTreeCore<()> as IntoSingleIds>::IntoIter;

    fn into_single_ids(self) -> Self::IntoIter {
        self.inner.into_single_ids()
    }
}

impl IterSingleIds for SpatialIdSet {
    type Iter<'a>
        = <FlexTreeCore<()> as IterSingleIds>::Iter<'a>
    where
        Self: 'a;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_single_ids()
    }
}
