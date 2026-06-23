use alloc::vec::Vec;

use crate::{
    CellValue, FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SingleId,
    SpatialIdTable,
};

impl<V> IntoFlexIds for SpatialIdTable<V>
where
    V: CellValue,
{
    type IntoIter = alloc::vec::IntoIter<FlexId>;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.iter()
            .map(|(sid, _)| FlexId::from(&sid))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V> IterFlexIds for SpatialIdTable<V>
where
    V: CellValue,
{
    type Iter<'a>
        = alloc::boxed::Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        alloc::boxed::Box::new(self.iter().map(|(sid, _)| FlexId::from(&sid)))
    }
}

impl<V> IntoSingleIds for SpatialIdTable<V>
where
    V: CellValue,
{
    type IntoIter = alloc::vec::IntoIter<SingleId>;

    fn into_single_ids(self) -> Self::IntoIter {
        self.iter()
            .map(|(sid, _)| sid)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V> IterSingleIds for SpatialIdTable<V>
where
    V: CellValue,
{
    type Iter<'a>
        = alloc::boxed::Box<dyn Iterator<Item = SingleId> + 'a>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        alloc::boxed::Box::new(self.iter().map(|(sid, _)| sid))
    }
}
