use std::collections::HashMap;
use std::hash::Hash;

use super::{Batch, KeyValueStore};

impl<K, V> KeyValueStore<K, V> for HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key).cloned()
    }

    fn batch_get(&self, keys: &[K]) -> Vec<Option<V>> {
        keys.iter().map(|key| self.get(key).cloned()).collect()
    }

    fn apply_batch(&mut self, batch: Batch<K, V>) {
        // メモリ実装なので、ループで適用
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
