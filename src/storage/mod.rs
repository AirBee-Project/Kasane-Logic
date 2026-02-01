use std::ops::RangeBounds;

pub mod btree_map;
pub mod hash_map;

/// 書き込み操作の塊
#[derive(Default)]
pub struct Batch<K, V> {
    pub puts: Vec<(K, V)>,
    pub deletes: Vec<K>,
}

impl<K, V> Batch<K, V> {
    pub fn new() -> Self {
        Self {
            puts: Vec::new(),
            deletes: Vec::new(),
        }
    }
    pub fn put(&mut self, key: K, value: V) {
        self.puts.push((key, value));
    }
    pub fn delete(&mut self, key: K) {
        self.deletes.push(key);
    }
}

pub trait KeyValueStore<K, V> {
    /// データへのアクセサ（参照など）
    /// ライフタイム 'a を戻り値に紐付けるための GAT
    type Accessor<'a>: std::ops::Deref<Target = V> where Self: 'a;

    /// 参照（アクセサ）を返すように変更
    fn get<'a>(&'a self, key: &K) -> Option<Self::Accessor<'a>>;
    fn batch_get<'a>(&'a self, keys: &[K]) -> Vec<Option<Self::Accessor<'a>>>;
    fn apply_batch(&mut self, batch: Batch<K, V>);
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a K, Self::Accessor<'a>)> + 'a>;
    fn len(&self) -> usize;
}

pub trait OrderedKeyValueStore<K, V>: KeyValueStore<K, V> {
    fn scan<'a, R>(&'a self, range: R) -> Box<dyn Iterator<Item = (&'a K, Self::Accessor<'a>)> + 'a>
    where
        R: RangeBounds<K>;

    fn last_key(&self) -> Option<K>;

    fn first_key(&self) -> Option<K>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_gats_zero_copy() {
        let mut store: BTreeMap<u64, String> = BTreeMap::new();
        store.insert(1, "hello".to_string());
        store.insert(2, "world".to_string());

        // Test that get() returns a reference accessor
        let value = store.get(&1).expect("Value should exist");
        assert_eq!(*value, "hello");
        
        // Test that the accessor implements Deref
        assert_eq!(value.len(), 5);
        
        // Test batch_get
        let values = store.batch_get(&[1, 2]);
        assert_eq!(values.len(), 2);
        assert_eq!(*values[0].as_ref().unwrap(), "hello");
        assert_eq!(*values[1].as_ref().unwrap(), "world");
        
        // Test iter
        let mut count = 0;
        for (key, value) in store.iter() {
            count += 1;
            match *key {
                1 => assert_eq!(*value, "hello"),
                2 => assert_eq!(*value, "world"),
                _ => panic!("Unexpected key"),
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ordered_store_scan() {
        let mut store: BTreeMap<u64, String> = BTreeMap::new();
        store.insert(1, "one".to_string());
        store.insert(2, "two".to_string());
        store.insert(3, "three".to_string());
        store.insert(4, "four".to_string());

        // Test scan with range
        let results: Vec<_> = store.scan(2..=3).collect();
        assert_eq!(results.len(), 2);
        assert_eq!(*results[0].0, 2);
        assert_eq!(*results[0].1, "two");
        assert_eq!(*results[1].0, 3);
        assert_eq!(*results[1].1, "three");
    }
}
