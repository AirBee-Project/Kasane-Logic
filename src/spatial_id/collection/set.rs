use std::{
    collections::BTreeMap,
    ops::{BitAnd, BitOr, Not, Sub},
};

use crate::spatial_id::{
    SpatialIdEncode,
    collection::{Rank, map::SpatialIdMap},
    encode::EncodeId,
    range::RangeId,
};

#[derive(Clone)]
pub struct SpatialIdSet {
    pub(crate) map: SpatialIdMap<()>,
}

impl SpatialIdSet {
    pub fn new() -> Self {
        Self {
            map: SpatialIdMap::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.map.size()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.map.keys()
    }

    fn iter_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.map.keys_encode()
    }

    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T) {
        self.map.insert(target, &());
    }

    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        self.map.remove(target);
    }

    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> SpatialIdSet {
        self.map.subset(target).to_set_join()
    }

    /// 和集合 (A | B)
    pub fn union(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result = self.clone();
        for encode in other.iter_encode() {
            result.insert(&encode);
        }
        result
    }

    /// 積集合 (A & B)
    pub fn intersection(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result = SpatialIdSet::new();
        let (small, large) = if self.size() < other.size() {
            (self, other)
        } else {
            (other, self)
        };

        for encode_id in small.iter_encode() {
            let related = large.map.related(&encode_id);
            for rank in related {
                if let Some((large_id, _)) = large.map.main.get(&rank) {
                    if let Some(inter) = encode_id.intersection(large_id) {
                        result.insert(&inter);
                    }
                }
            }
        }
        result
    }

    /// 差集合 (A - B)
    pub fn difference(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result = self.clone();
        for encode in other.iter_encode() {
            result.remove(&encode);
        }
        result
    }
}

// 演算子オーバーロード

impl BitOr for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitAnd for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for &SpatialIdSet {
    type Output = SpatialIdSet;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}

impl Not for SpatialIdSet {
    type Output = Self;
    fn not(self) -> Self::Output {
        let mut universe = SpatialIdSet::new();
        let root_range = unsafe { RangeId::new_unchecked(0, [0, 1], [0, 0], [0, 0]) };
        universe.insert(&root_range);
        universe.difference(&self)
    }
}

// イテレータ実装

impl IntoIterator for SpatialIdSet {
    type Item = RangeId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.map.main.into_values().map(|(id, _)| id.decode()))
    }
}

impl<'a> IntoIterator for &'a SpatialIdSet {
    type Item = RangeId;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.map.keys())
    }
}
