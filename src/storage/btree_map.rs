use std::collections::BTreeMap;

use crate::{KeyValueStore, OrderedKeyValueStore, storage::Batch};

impl<K, V> KeyValueStore<K, V> for BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key).cloned()
    }

    fn batch_get(&self, keys: &[K]) -> Vec<Option<V>> {
        keys.iter().map(|key| self.get(key).cloned()).collect()
    }

    fn apply_batch(&mut self, batch: Batch<K, V>) {
        for key in batch.deletes {
            self.remove(&key);
        }
        for (key, value) in batch.puts {
            self.insert(key, value);
        }
    }
    fn iter(&self) -> impl Iterator<Item = (K, V)> {
        Box::new(self.iter().map(|(k, v)| (k.clone(), v.clone())))
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
    fn scan<R>(&self, range: R) -> Box<dyn Iterator<Item = (K, V)> + '_>
    where
        R: RangeBounds<K>,
    {
        Box::new(self.range(range).map(|(k, v)| (k.clone(), v.clone())))
    }

    fn last_key(&self) -> Option<K> {
        self.keys().next_back().cloned()
    }

    fn first_key(&self) -> Option<K> {
        self.keys().next().cloned()
    }
}
