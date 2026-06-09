use crate::{
    FlexTreeCore, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SpatialIdMap,
    SpatialIdSet, SpatialIdTable,
};

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

impl<V> From<&SpatialIdTable<V>> for SpatialIdSet
where
    V: PartialEq + Ord + Clone,
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
    V: PartialEq + Ord + Clone,
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

impl<V> From<&SpatialIdMap<V>> for SpatialIdSet
where
    V: PartialEq + Clone,
{
    /// 値を捨て、占有空間だけを [`SpatialIdSet`] へ写し取る。元のマップは消費しない。
    fn from(map: &SpatialIdMap<V>) -> Self {
        let mut set = SpatialIdSet::new();
        for flex_id in map.iter_flex_ids() {
            set.insert(flex_id);
        }
        set
    }
}

impl<V> From<SpatialIdMap<V>> for SpatialIdSet
where
    V: PartialEq + Clone,
{
    /// [`SpatialIdMap`] を、値を捨てて占有空間だけを持つ [`SpatialIdSet`] へ変換する。
    ///
    /// # 動作例
    ///
    /// マップから集合へ:
    /// ```
    /// use kasane_logic::{SingleId, SpatialIdMap, SpatialIdSet};
    /// let mut map: SpatialIdMap<&str> = SpatialIdMap::new();
    /// map.insert(SingleId::new(20, 0, 0, 0).unwrap(), "a");
    ///
    /// let set = SpatialIdSet::from(map);
    /// assert!(set.get(&SingleId::new(20, 0, 0, 0).unwrap()).next().is_some());
    /// ```
    fn from(map: SpatialIdMap<V>) -> Self {
        Self::from(&map)
    }
}
