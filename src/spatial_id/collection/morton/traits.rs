//! Morton バックエンドの [`SpatialIdCollection`] 実装。
//!
//! 公開 API は [`SingleId`](crate::SingleId) を返すが、`expr` エンジンが要求する
//! このトレイトは [`FlexId`] ベースなので、`SingleId ⇆ FlexId` 変換で橋渡しする。

use crate::{CellValue, FlexId, SpatialIdCollection, SpatialIdSet, SpatialIdTable};

/// `SpatialIdSet` の参照走査で返す `&()` の実体。
static UNIT: () = ();

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: CellValue,
{
    type Value = V;

    fn empty() -> Self {
        SpatialIdTable::new()
    }

    fn insert(&mut self, key: FlexId, value: V) {
        SpatialIdTable::insert(self, key, value);
    }

    fn scan(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        self.iter().map(|(sid, v)| (FlexId::from(&sid), v.clone()))
    }

    fn scan_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.iter().map(|(sid, v)| (FlexId::from(&sid), v))
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, V)> + 'a {
        self.get(target)
            .map(|(sid, v)| (FlexId::from(&sid), v.clone()))
    }

    fn query_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.get(target).map(|(sid, v)| (FlexId::from(&sid), v))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdTable::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdTable::is_empty(self)
    }
}

impl SpatialIdCollection for SpatialIdSet {
    type Value = ();

    fn empty() -> Self {
        SpatialIdSet::new()
    }

    fn insert(&mut self, key: FlexId, _value: ()) {
        SpatialIdSet::insert(self, key);
    }

    fn scan(&self) -> impl Iterator<Item = (FlexId, ())> + '_ {
        self.iter().map(|sid| (FlexId::from(&sid), ()))
    }

    fn scan_ref(&self) -> impl Iterator<Item = (FlexId, &())> + '_ {
        self.iter().map(|sid| (FlexId::from(&sid), &UNIT))
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, ())> + 'a {
        self.get(target).map(|sid| (FlexId::from(&sid), ()))
    }

    fn query_ref<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, &'a ())> + 'a {
        self.get(target).map(|sid| (FlexId::from(&sid), &UNIT))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdSet::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdSet::is_empty(self)
    }
}
