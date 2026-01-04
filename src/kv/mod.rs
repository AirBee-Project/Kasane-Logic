//Kasaneがバックエンドに求める能力をTraitとして定義したもの

use std::ops::RangeBounds;

pub trait MapIter<V> {
    type Iter: Iterator<Item = V>;
    fn iter(&self) -> Self::Iter;
}

pub trait MapGet<K, V>
where
    K: Ord,
{
    fn get(&self, key: &K) -> Option<V>;
}

pub trait MapRange<K, V>
where
    K: Ord,
{
    type RangeIter: Iterator<Item = V>;

    fn range<R>(&self, range: R) -> Self::RangeIter
    where
        R: RangeBounds<K>;
}

pub trait MapInsert<K, V>
where
    K: Ord,
{
    fn insert(&mut self, key: K, value: V);
}

pub trait MapRemove<K>
where
    K: Ord,
{
    fn remove(&mut self, key: &K);
}

pub trait MapUpdate<K, V>
where
    K: Ord,
{
    fn update<F>(&mut self, key: &K, f: F)
    where
        F: FnOnce(&V) -> V;
}
