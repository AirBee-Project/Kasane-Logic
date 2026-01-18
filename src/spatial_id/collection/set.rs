use roaring::RoaringTreemap;
use std::{
    collections::{BTreeMap, btree_map::Range},
    ops::{BitAnd, BitOr, Not, Sub},
};

use crate::spatial_id::{
    SpatialIdEncode,
    encode::{self, EncodeId},
    range::RangeId,
    segment::encode::EncodeSegment,
};
use std::ops::Bound::{Excluded, Included};

type Rank = u64;

#[derive(Clone)]
pub struct SpatialIdSet {
    f: BTreeMap<EncodeSegment, RoaringTreemap>,
    x: BTreeMap<EncodeSegment, RoaringTreemap>,
    y: BTreeMap<EncodeSegment, RoaringTreemap>,
    main: BTreeMap<Rank, EncodeId>,
    next_rank: Rank,
}

impl SpatialIdSet {
    ///新しく[SpatialIdSet]を作成する。
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
        }
    }

    ///[SpatialIdSet]に含まれる[RangeId]を取り出す。
    pub fn iter(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.decode())
    }

    ///[SpatialIdSet]に含まれる[EncodeId]を取り出す。
    fn iter_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.clone())
    }

    ///[SpatialIdSet]に入っている[EncodeId]の個数を返す。
    pub fn size(&self) -> usize {
        self.main.len()
    }

    ///[SpatialIdSet]の中にある`target`と関連ある[EncodeId]のRankを返す。
    fn related(&self, target: &EncodeId) -> RoaringTreemap {
        let related_segments = |map: &BTreeMap<EncodeSegment, RoaringTreemap>,
                                target_seg: &EncodeSegment|
         -> RoaringTreemap {
            let mut related_bitmap = RoaringTreemap::new();

            let mut current = target_seg.parent();
            while let Some(seg) = current {
                if let Some(ranks) = map.get(&seg) {
                    related_bitmap |= ranks;
                }
                current = seg.parent();
            }

            let range_end = target_seg.descendant_range_end();
            for (_, ranks) in map.range((Included(target_seg), Excluded(&range_end))) {
                related_bitmap |= ranks;
            }

            related_bitmap
        };

        let f_related = related_segments(&self.f, target.as_f());
        let x_related = related_segments(&self.x, target.as_x());
        let y_related = related_segments(&self.y, target.as_y());

        // 3次元すべての積集合をとる
        f_related & x_related & y_related
    }

    pub unsafe fn insert_unchecked<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            let rank = self.next_rank;
            self.next_rank += 1;

            self.f
                .entry(encode_id.as_f().clone())
                .or_default()
                .insert(rank);
            self.x
                .entry(encode_id.as_x().clone())
                .or_default()
                .insert(rank);
            self.y
                .entry(encode_id.as_y().clone())
                .or_default()
                .insert(rank);
            self.main.insert(rank, encode_id.clone());
        }
    }

    ///[SpatialIdSet]に[SpatialId]を挿入する。
    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            let mut is_contained = false;
            let mut to_remove = Vec::new();
            for rank in related_ranks {
                if let Some(existing_id) = self.main.get(&rank) {
                    if existing_id.contains(&encode_id) {
                        is_contained = true;
                        break;
                    } else if encode_id.contains(existing_id) {
                        to_remove.push(rank);
                    }
                }
            }
            if is_contained {
                continue;
            }
            for rank in to_remove {
                self.remove_rank(rank);
            }
            unsafe { self.join_insert_unchecked(&encode_id) };
        }
    }

    ///対象のID領域を削除する。
    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            for related_rank in self.related(&encode_id) {
                let base = self.main.remove(&related_rank).unwrap();
                self.f.remove(base.as_f());
                self.x.remove(base.as_x());
                self.y.remove(base.as_y());
                let diff = base.difference(&encode_id);
                for need_insert in diff {
                    unsafe { self.join_insert_unchecked(&need_insert) }
                }
            }
        }
    }

    ///[SpatialIdSet]同士の差集合を作成する。
    pub fn difference(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result;
        if self.size() < other.size() {
            result = other.clone();
            for encode in self.iter_encode() {
                result.remove(&encode);
            }
        } else {
            result = self.clone();
            for encode in other.iter_encode() {
                result.remove(&encode);
            }
        };
        result
    }

    ///[SpatialIdSet]の特定の[SpatialId]と重なる部分だけを取り出す。
    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> SpatialIdSet {
        let mut result = SpatialIdSet::new();
        for encode_id in target.encode() {
            for related_rank in self.related(&encode_id) {
                let base = self.main.get(&related_rank).unwrap();
                let intersection = encode_id.intersection(base).unwrap();
                unsafe { result.join_insert_unchecked(&intersection) };
            }
        }
        result
    }

    ///[SpatialIdSet]同士の積集合を作成する。
    pub fn intersection(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result = SpatialIdSet::new();
        let large;
        let small;

        if self.size() < other.size() {
            large = other;
            small = self;
        } else {
            large = self;
            small = other;
        };

        for encode_id in small.iter_encode() {
            for intersecton_encode_id in large.subset(&encode_id).iter_encode() {
                unsafe { result.join_insert_unchecked(&intersecton_encode_id) };
            }
        }

        result
    }

    ///[SpatialIdSet]同士の和集合を作成する。
    pub fn union(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result;
        if self.size() < other.size() {
            result = other.clone();
            for encode in self.iter_encode() {
                result.insert(&encode);
            }
        } else {
            result = self.clone();
            for encode in other.iter_encode() {
                result.insert(&encode);
            }
        };
        result
    }

    /// IDを追加し、可能な場合は結合を行う。
    /// 重複チェックは行っていないので、その責任は関数の使用者が負う。
    pub unsafe fn join_insert_unchecked<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            //Fの兄弟候補
            let f_sibling = EncodeId::new(
                encode_id.as_f().sibling(),
                encode_id.as_x().clone(),
                encode_id.as_y().clone(),
            );

            //Fで結合
            match self.find_encode(&f_sibling) {
                Some(rank) => {
                    self.remove_rank(rank);
                    unsafe {
                        self.join_insert_unchecked(&EncodeId::new(
                            encode_id.as_f().parent().unwrap(),
                            encode_id.as_x().clone(),
                            encode_id.as_y().clone(),
                        ))
                    };
                    return;
                }
                None => {}
            }

            //Xの兄弟候補
            let x_sibling = EncodeId::new(
                encode_id.as_f().clone(),
                encode_id.as_x().sibling(),
                encode_id.as_y().clone(),
            );

            //Xで結合
            match self.find_encode(&x_sibling) {
                Some(rank) => {
                    self.remove_rank(rank);
                    unsafe {
                        self.join_insert_unchecked(&EncodeId::new(
                            encode_id.as_f().clone(),
                            encode_id.as_x().parent().unwrap(),
                            encode_id.as_y().clone(),
                        ))
                    };
                    return;
                }
                None => {}
            }

            //Yの兄弟候補
            let y_sibling = EncodeId::new(
                encode_id.as_f().clone(),
                encode_id.as_x().clone(),
                encode_id.as_y().sibling(),
            );

            //Yで結合
            match self.find_encode(&y_sibling) {
                Some(rank) => {
                    self.remove_rank(rank);
                    unsafe {
                        self.join_insert_unchecked(&EncodeId::new(
                            encode_id.as_f().clone(),
                            encode_id.as_x().clone(),
                            encode_id.as_y().parent().unwrap(),
                        ))
                    };
                    return;
                }
                None => {}
            }

            unsafe { self.insert_unchecked(target) };
        }
    }

    ///指定されたEncodeIdと完全に一致するEncodeIdのRankを返す。
    fn find_encode(&self, target: &EncodeId) -> Option<Rank> {
        let f_hits = self.f.get(target.as_f())?;
        let x_hits = self.x.get(target.as_x())?;
        let y_hits = self.y.get(target.as_y())?;
        let result = f_hits & x_hits & y_hits;
        result.iter().next()
    }

    /// 指定されたRankを持つIDを全てのインデックスから完全に削除する。
    fn remove_rank(&mut self, rank: Rank) {
        let encode_id = match self.main.remove(&rank) {
            Some(v) => v,
            None => return,
        };

        let remove_from_dim = |map: &mut BTreeMap<EncodeSegment, RoaringTreemap>,
                               segment: EncodeSegment| {
            if let std::collections::btree_map::Entry::Occupied(mut entry) = map.entry(segment) {
                let bitmap = entry.get_mut();
                bitmap.remove(rank);
                if bitmap.is_empty() {
                    entry.remove_entry();
                }
            }
        };

        remove_from_dim(&mut self.f, encode_id.as_f().clone());
        remove_from_dim(&mut self.x, encode_id.as_x().clone());
        remove_from_dim(&mut self.y, encode_id.as_y().clone());
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.main.is_empty()
    }

    /// 全削除
    pub fn clear(&mut self) {
        self.f.clear();
        self.x.clear();
        self.y.clear();
        self.main.clear();
        self.next_rank = 0;
    }
}

impl BitOr for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

// A & B (積集合)
impl BitAnd for &SpatialIdSet {
    type Output = SpatialIdSet;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

// A - B (差集合)
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
        let root_range = unsafe { RangeId::new_unchecked(0, [0, 1], [0, 1], [0, 1]) };
        universe.insert(&root_range);
        universe.difference(&self)
    }
}

impl<T: SpatialIdEncode> FromIterator<T> for SpatialIdSet {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = SpatialIdSet::new();
        for item in iter {
            set.insert(&item);
        }
        set
    }
}

impl<T: SpatialIdEncode> Extend<T> for SpatialIdSet {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(&item);
        }
    }
}

impl Default for SpatialIdSet {
    fn default() -> Self {
        Self::new()
    }
}
