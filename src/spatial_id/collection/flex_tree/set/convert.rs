use crate::spatial_id::collection::flex_tree::core::Summary;
use crate::{SpatialIdSet, SpatialIdTable};

impl<V, S> From<&SpatialIdTable<V, S>> for SpatialIdSet
where
    V: PartialEq + Clone,
    S: Summary<V>,
{
    /// 値を捨て、占有空間だけを [`SpatialIdSet`] へ写し取る。元のテーブルは消費しない。
    fn from(table: &SpatialIdTable<V, S>) -> Self {
        table.iter().map(|(flex_id, _)| flex_id).collect()
    }
}

impl<V, S> From<SpatialIdTable<V, S>> for SpatialIdSet
where
    V: PartialEq + Clone,
    S: Summary<V>,
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
    /// assert!(set.get(SingleId::new(20, 0, 0, 0).unwrap()).next().is_some());
    /// assert!(set.get(SingleId::new(20, 5, 0, 0).unwrap()).next().is_some());
    /// assert!(set.get(SingleId::new(20, 9, 0, 0).unwrap()).next().is_none());
    /// ```
    fn from(table: SpatialIdTable<V, S>) -> Self {
        Self::from(&table)
    }
}
