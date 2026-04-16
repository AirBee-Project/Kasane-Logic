use crate::{FlexTreeCore, FlexTreeMap, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds};

impl<V> IntoFlexIds for FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    type IntoIter = <FlexTreeCore<V> as IntoFlexIds>::IntoIter;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.inner.into_flex_ids()
    }
}

impl<V> IterFlexIds for FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    type Iter<'a>
        = <FlexTreeCore<V> as IterFlexIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_flex_ids()
    }
}

impl<V> IntoSingleIds for FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    type IntoIter = <FlexTreeCore<V> as IntoSingleIds>::IntoIter;

    fn into_single_ids(self) -> Self::IntoIter {
        self.inner.into_single_ids()
    }
}

impl<V> IterSingleIds for FlexTreeMap<V>
where
    V: PartialEq + Clone,
{
    type Iter<'a>
        = <FlexTreeCore<V> as IterSingleIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_single_ids()
    }
}
