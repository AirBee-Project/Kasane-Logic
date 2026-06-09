#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{FlexId, SpatialIdSet, SpatialIdTable};

/// 演算の対象となる空間IDコレクションの性質。
///
/// `SpatialIdTable` / `SpatialIdSet` を演算子から一様に扱うための抽象。
/// 「FlexId をキーに値を持つコレクション」であればよく、値を持たない集合（Set）は
/// `Value = ()` とする。
pub trait SpatialIdCollection: Sized {
    /// 各空間IDに紐づく値の型。値を持たない集合では `()`。
    type Value: Ord + PartialEq + Clone;

    /// 空のコレクションを作る（演算結果の組み立て先）。
    fn empty() -> Self;

    /// 1 つの `(FlexId, Value)` を書き込む。
    fn insert(&mut self, key: FlexId, value: Self::Value);

    /// 保持している全ての `(FlexId, Value)` を走査する。
    fn scan(&self) -> impl Iterator<Item = (FlexId, Self::Value)> + '_;

    /// `target` と重なる `(FlexId, Value)` を取得する（2項演算の重なり判定に使う）。
    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, Self::Value)> + 'a;

    /// 含まれる要素の最大ズームレベル（正規化・最適化に使う）。
    fn max_zoomlevel(&self) -> Option<u8>;

    /// 空かどうか。
    fn is_empty(&self) -> bool;
}

impl<V> SpatialIdCollection for SpatialIdTable<V>
where
    V: Ord + PartialEq + Clone,
{
    type Value = V;

    fn empty() -> Self {
        SpatialIdTable::new()
    }

    fn insert(&mut self, key: FlexId, value: V) {
        SpatialIdTable::insert(self, key, value);
    }

    fn scan(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        self.iter().map(|(id, v)| (id, v.clone()))
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, V)> + 'a {
        self.get(target).map(|(id, v)| (id, v.clone()))
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
        self.iter().map(|id| (id, ()))
    }

    fn query<'a>(&'a self, target: &'a FlexId) -> impl Iterator<Item = (FlexId, ())> + 'a {
        self.get(target).map(|id| (id, ()))
    }

    fn max_zoomlevel(&self) -> Option<u8> {
        SpatialIdSet::max_zoomlevel(self)
    }

    fn is_empty(&self) -> bool {
        SpatialIdSet::is_empty(self)
    }
}
