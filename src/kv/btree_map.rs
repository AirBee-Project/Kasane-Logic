use std::{collections::BTreeMap, ops::RangeBounds};

use crate::kv::KvStore;

impl<K, V> KvStore<K, V> for BTreeMap<K, V>
where
    K: Ord,
{
    type Iter<'a>
        = std::collections::btree_map::Iter<'a, K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    type RangeIter<'a>
        = std::collections::btree_map::Range<'a, K, V>
    where
        Self: 'a,
        K: 'a,
        V: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.iter()
    }

    fn range<R>(&self, range: R) -> Self::RangeIter<'_>
    where
        R: RangeBounds<K>,
    {
        self.range(range)
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    fn update<F>(&mut self, key: &K, f: F)
    where
        F: FnOnce(&mut V),
    {
        if let Some(v) = self.get_mut(key) {
            f(v);
        }
    }

    fn len(&self) -> usize {
        self.len()
    }
}
