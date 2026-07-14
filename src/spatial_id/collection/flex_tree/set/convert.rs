use crate::{FlexId, SingleId, SpatialIdSet, SpatialIdTable};

impl SpatialIdSet {
    pub fn flex_ids(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter().map(|(flex_id, _)| flex_id)
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.inner.single_ids()
    }
}

impl<V> From<&SpatialIdTable<V>> for SpatialIdSet
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue + Ord,
{
    /// 値を捨て、占有空間だけを [`SpatialIdSet`] へ写し取る。元のテーブルは消費しない。
    fn from(table: &SpatialIdTable<V>) -> Self {
        let mut set = SpatialIdSet::new();
        for flex_id in table.flex_ids() {
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
