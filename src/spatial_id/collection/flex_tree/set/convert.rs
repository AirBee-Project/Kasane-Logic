use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    FlexId, IterFlexIds, IterSingleIds, SingleId, SpatialIdSet,
    SpatialIdTable,
};



impl IterFlexIds for SpatialIdSet {
    type Iter<'a>
        = Box<dyn Iterator<Item = FlexId> + 'a>
    where
        Self: 'a;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        Box::new(self.iter())
    }
}



impl IterSingleIds for SpatialIdSet {
    type Iter<'a>
        = alloc::vec::IntoIter<SingleId>
    where
        Self: 'a;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        let ids: Vec<SingleId> = self.flat_single_ids().collect();
        ids.into_iter()
    }
}

impl<V> From<&SpatialIdTable<V>> for SpatialIdSet
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// 値を捨て、占有空間だけを [`SpatialIdSet`] へ写し取る。元のテーブルは消費しない。
    fn from(table: &SpatialIdTable<V>) -> Self {
        let mut set = SpatialIdSet::new();
        for flex_id in table.iter_flex_ids() {
            set.insert(flex_id);
        }
        set
    }
}

impl<V> From<SpatialIdTable<V>> for SpatialIdSet
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// [`SpatialIdTable`] を、値を捨てて占有空間だけを持つ [`SpatialIdSet`] へ変換する。
    ///
    /// # 動作例
    ///
    /// 値付きテーブルから集合へ:
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdSet, SpatialIdTable};
    /// let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
    /// table.insert(SingleId::new(20, 0, 0, 0).unwrap(), 7);
    /// table.insert(SingleId::new(20, 5, 0, 0).unwrap(), 9);
    ///
    /// let set = SpatialIdSet::from(table);
    /// assert!(set.get(&SingleId::new(20, 0, 0, 0).unwrap()).next().is_some());
    /// assert!(set.get(&SingleId::new(20, 5, 0, 0).unwrap()).next().is_some());
    /// assert!(set.get(&SingleId::new(20, 9, 0, 0).unwrap()).next().is_none());
    /// ```
    fn from(table: SpatialIdTable<V>) -> Self {
        Self::from(&table)
    }
}
