use std::ops::RangeBounds;

pub mod btree_map;

pub trait BTreeMapTrait<K, V>
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

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn range<R>(&self, range: R) -> Self::RangeIter<'_>
    where
        R: RangeBounds<K>;

    fn get(&self, key: &K) -> Option<&V>;

    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn remove(&mut self, key: &K) -> Option<V>;

    fn update<F>(&mut self, key: &K, f: F)
    where
        F: FnOnce(&mut V);

    fn get_or_insert_with<F>(&mut self, key: K, f: F) -> &mut V
    where
        F: FnOnce() -> V;

    fn clear(&mut self);

    type IterMut<'a>: Iterator<Item = (&'a K, &'a mut V)>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    fn iter_mut(&mut self) -> Self::IterMut<'_>;

    fn first_key_value(&self) -> Option<(&K, &V)>;
    fn last_key_value(&self) -> Option<(&K, &V)>;
}
