#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{FlexTreeCore, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialIdTable};

impl<V> IntoFlexIds for SpatialIdTable<V>
where
    V: PartialEq + Clone + Ord,
{
    type IntoIter = <FlexTreeCore<V> as IntoFlexIds>::IntoIter;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.inner.into_flex_ids()
    }
}

impl<V> IterFlexIds for SpatialIdTable<V>
where
    V: PartialEq + Clone + Ord,
{
    type Iter<'a>
        = <FlexTreeCore<V> as IterFlexIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_flex_ids()
    }
}

impl<V> IntoSingleIds for SpatialIdTable<V>
where
    V: PartialEq + Clone + Ord,
{
    type IntoIter = <FlexTreeCore<V> as IntoSingleIds>::IntoIter;

    fn into_single_ids(self) -> Self::IntoIter {
        self.inner.into_single_ids()
    }
}

impl<V> IterSingleIds for SpatialIdTable<V>
where
    V: PartialEq + Clone + Ord,
{
    type Iter<'a>
        = <FlexTreeCore<V> as IterSingleIds>::Iter<'a>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        self.inner.iter_single_ids()
    }
}
