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
