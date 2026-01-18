use std::ops::RangeBounds;

pub mod btree_map;

pub trait KvStore<K, V>
where
    K: Ord,
{
    type Iter<'a>: Iterator<Item = (&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type RangeIter<'a>: Iterator<Item = (&'a K, &'a V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    fn iter(&self) -> Self::Iter<'_>;

    fn range<R>(&self, range: R) -> Self::RangeIter<'_>
    where
        R: RangeBounds<K>;

    fn get(&self, key: &K) -> Option<&V>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn remove(&mut self, key: &K) -> Option<V>;

    fn update<F>(&mut self, key: &K, f: F)
    where
        F: FnOnce(&mut V);
}
