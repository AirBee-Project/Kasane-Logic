use std::collections::BTreeMap;

use crate::{KeyValueStore, OrderedKeyValueStore, storage::Batch};

impl<K, V> KeyValueStore<K, V> for BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    type Accessor<'a> = &'a V where Self: 'a;

    fn get<'a>(&'a self, key: &K) -> Option<Self::Accessor<'a>> {
        self.get(key)
    }

    fn batch_get<'a>(&'a self, keys: &[K]) -> Vec<Option<Self::Accessor<'a>>> {
        keys.iter().map(|key| self.get(key)).collect()
    }

    fn apply_batch(&mut self, batch: Batch<K, V>) {
        for key in batch.deletes {
            self.remove(&key);
        }
        for (key, value) in batch.puts {
            self.insert(key, value);
        }
    }
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a K, Self::Accessor<'a>)> + 'a> {
        Box::new(self.iter())
    }

    fn len(&self) -> usize {
        self.len()
    }
}

use std::ops::RangeBounds;

impl<K, V> OrderedKeyValueStore<K, V> for BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn scan<'a, R>(&'a self, range: R) -> Box<dyn Iterator<Item = (&'a K, Self::Accessor<'a>)> + 'a>
    where
        R: RangeBounds<K>,
    {
        Box::new(self.range(range))
    }

    fn last_key(&self) -> Option<K> {
        self.keys().next_back().cloned()
    }

    fn first_key(&self) -> Option<K> {
        self.keys().next().cloned()
    }
}
