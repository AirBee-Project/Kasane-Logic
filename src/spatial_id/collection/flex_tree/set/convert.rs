use crate::{FlexTreeCore, FlexTreeSet, IntoFlexIds, IterFlexIds};

impl IntoFlexIds for FlexTreeSet {
    type IntoIter = <FlexTreeCore<()> as IntoFlexIds>::IntoIter;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.inner.into_flex_ids()
    }
}

impl IterFlexIds for FlexTreeSet {
    type Iter<'a>
        = <FlexTreeCore<()> as IterFlexIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_flex_ids()
    }
}
