use alloc::vec::Vec;

use crate::{
    FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SingleId, SpatialIdSet,
    SpatialIdTable,
};

impl IntoFlexIds for SpatialIdSet {
    type IntoIter = alloc::vec::IntoIter<FlexId>;

    fn into_flex_ids(self) -> Self::IntoIter {
        self.iter()
            .map(|sid| FlexId::from(&sid))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl IterFlexIds for SpatialIdSet {
    type Iter<'a>
        = alloc::boxed::Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        alloc::boxed::Box::new(self.iter().map(|sid| FlexId::from(&sid)))
    }
}

impl IntoSingleIds for SpatialIdSet {
    type IntoIter = alloc::vec::IntoIter<SingleId>;

    fn into_single_ids(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl IterSingleIds for SpatialIdSet {
    type Iter<'a>
        = alloc::boxed::Box<dyn Iterator<Item = SingleId> + 'a>
    where
        Self: 'a;

    fn iter_single_ids(&self) -> Self::Iter<'_> {
        alloc::boxed::Box::new(self.iter())
    }
}

impl<V> From<&SpatialIdTable<V>> for SpatialIdSet
where
    V: crate::CellValue,
{
    /// 値を捨て、占有空間だけを [`SpatialIdSet`] へ写し取る。
    fn from(table: &SpatialIdTable<V>) -> Self {
        let mut set = SpatialIdSet::new();
        for sid in table.iter().map(|(sid, _)| sid) {
            set.insert(sid);
        }
        set
    }
}

impl<V> From<SpatialIdTable<V>> for SpatialIdSet
where
    V: crate::CellValue,
{
    fn from(table: SpatialIdTable<V>) -> Self {
        Self::from(&table)
    }
}
