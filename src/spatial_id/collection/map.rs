use std::{
    cell::Cell,
    collections::BTreeMap,
    ops::{
        Bound::{Excluded, Included},
        Deref, DerefMut,
    },
};

use roaring::RoaringTreemap;

use crate::{
    RangeId,
    kv::KvStore,
    spatial_id::{
        SpatialIdEncode,
        collection::{MapTrait, Rank},
        flex_id::FlexId,
        segment::Segment,
    },
};

// =============================================================================
//  1. Public API: SpatialIdMap (完全体)
// =============================================================================

#[derive(Clone)]
pub struct SpatialIdMap<V>(MapLogic<OnMemoryMap<V>>);

impl<V> SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    pub fn new() -> Self {
        Self(MapLogic::new(OnMemoryMap::new()))
    }
}

impl<V> Default for SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Deref for SpatialIdMap<V> {
    type Target = MapLogic<OnMemoryMap<V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for SpatialIdMap<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// --- Iterator Implementation ---

impl<V> IntoIterator for SpatialIdMap<V> {
    type Item = (RangeId, V);
    type IntoIter = MapIntoIter<V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, V> IntoIterator for &'a SpatialIdMap<V> {
    type Item = (RangeId, &'a V);
    type IntoIter = MapIter<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, V> IntoIterator for &'a mut SpatialIdMap<V> {
    type Item = (RangeId, &'a mut V);
    type IntoIter = MapIterMut<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// --- Collection Traits ---

impl<V> FromIterator<(RangeId, V)> for SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    fn from_iter<I: IntoIterator<Item = (RangeId, V)>>(iter: I) -> Self {
        let mut map = SpatialIdMap::new();
        map.extend(iter);
        map
    }
}

impl<V> Extend<(RangeId, V)> for SpatialIdMap<V>
where
    V: Clone + PartialEq,
{
    fn extend<I: IntoIterator<Item = (RangeId, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(&key, &value);
        }
    }
}

// =============================================================================
//  2. Internal Logic: MapLogic
// =============================================================================

#[derive(Clone)]
pub struct MapLogic<S>(S);

// イテレータの型エイリアス
// ここで fn pointer を指定しているため、実装側でもクロージャではなく関数を使うかキャストが必要
pub type MapIter<'a, V> = std::iter::Map<
    std::collections::btree_map::Iter<'a, Rank, (FlexId, V)>,
    fn((&'a Rank, &'a (FlexId, V))) -> (RangeId, &'a V),
>;

pub type MapIterMut<'a, V> = std::iter::Map<
    std::collections::btree_map::IterMut<'a, Rank, (FlexId, V)>,
    fn((&'a Rank, &'a mut (FlexId, V))) -> (RangeId, &'a mut V),
>;

pub type MapIntoIter<V> = std::iter::Map<
    std::collections::btree_map::IntoIter<Rank, (FlexId, V)>,
    fn((Rank, (FlexId, V))) -> (RangeId, V),
>;

// -----------------------------------------------------------------------------
//  Generic Implementation (S: MapTrait)
//  どのストレージでも共通して使えるロジック
// -----------------------------------------------------------------------------
impl<S> MapLogic<S>
where
    S: MapTrait,
{
    pub fn new(store: S) -> Self {
        Self(store)
    }
    pub fn size(&self) -> usize {
        self.0.main().len()
    }
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn inner(&self) -> &S {
        &self.0
    }
    pub(crate) fn inner_mut(&mut self) -> &mut S {
        &mut self.0
    }

    // キーのイテレータ (ジェネリックなまま実装可能)
    pub fn keys(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.main().iter().map(|(_, (id, _))| id.decode())
    }

    pub fn keys_encode(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.0.main().iter().map(|(_, (id, _))| id.clone())
    }

    pub fn values(&self) -> impl Iterator<Item = &S::V> + '_ {
        self.0.main().iter().map(|(_, (_, v))| v)
    }
}

// -----------------------------------------------------------------------------
//  Concrete Implementation (S = OnMemoryMap<V>)
//  具体的なストレージ(BTreeMap)を知っているため、型エイリアスに合わせたイテレータを返せる
// -----------------------------------------------------------------------------
impl<V> MapLogic<OnMemoryMap<V>> {
    // マッピング用の関数（クロージャではなく関数として定義して型を合わせる）
    fn map_ref<'a>((_, (id, v)): (&'a Rank, &'a (FlexId, V))) -> (RangeId, &'a V) {
        (id.decode(), v)
    }

    fn map_mut<'a>((_, (id, v)): (&'a Rank, &'a mut (FlexId, V))) -> (RangeId, &'a mut V) {
        (id.decode(), v)
    }

    fn map_owned((_, (id, v)): (Rank, (FlexId, V))) -> (RangeId, V) {
        (id.decode(), v)
    }

    pub fn iter(&self) -> MapIter<'_, V> {
        self.0.main.iter().map(Self::map_ref)
    }

    pub fn iter_mut(&mut self) -> MapIterMut<'_, V> {
        self.0.main.iter_mut().map(Self::map_mut)
    }

    pub fn into_iter(self) -> MapIntoIter<V> {
        self.0.main.into_iter().map(Self::map_owned)
    }
}

// -----------------------------------------------------------------------------
//  Generic Logic Implementation (Insert/Remove)
// -----------------------------------------------------------------------------
impl<S> MapLogic<S>
where
    S: MapTrait + Default,
    S::V: Clone + PartialEq,
{
    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T, value: &S::V) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            let mut to_process = Vec::new();
            let mut is_completely_covered = false;

            for rank in related_ranks {
                if let Some((existing_id, existing_val)) = self.0.main().get(&rank) {
                    if existing_id.contains(&encode_id) {
                        if existing_val == value {
                            is_completely_covered = true;
                            break;
                        } else {
                            to_process.push((rank, "SPLIT"));
                        }
                    } else if encode_id.contains(existing_id) {
                        to_process.push((rank, "REMOVE"));
                    }
                }
            }

            if is_completely_covered {
                continue;
            }

            for (rank, action) in to_process {
                if let Some((old_id, old_val)) = self.remove_rank(rank) {
                    if action == "SPLIT" {
                        let diff = old_id.difference(&encode_id);
                        for piece in diff {
                            unsafe { self.join_insert_unchecked(&piece, &old_val) };
                        }
                    }
                }
            }
            unsafe { self.join_insert_unchecked(&encode_id, value) };
        }
    }

    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            let ranks: Vec<Rank> = related_ranks.into_iter().collect();

            for rank in ranks {
                if let Some((existing_id, existing_val)) = self.remove_rank(rank) {
                    let diff = existing_id.difference(&encode_id);
                    for piece in diff {
                        unsafe { self.join_insert_unchecked(&piece, &existing_val) };
                    }
                }
            }
        }
    }

    pub fn subset<T: SpatialIdEncode>(&self, target: &T) -> MapLogic<S> {
        let mut result_map = MapLogic::new(S::default());
        for encode_id in target.encode() {
            let related_ranks = self.related(&encode_id);
            for rank in related_ranks {
                if let Some((existing_id, existing_val)) = self.0.main().get(&rank) {
                    if let Some(intersection) = encode_id.intersection(existing_id) {
                        unsafe { result_map.join_insert_unchecked(&intersection, existing_val) };
                    }
                }
            }
        }
        result_map
    }

    pub(crate) fn related(&self, target: &FlexId) -> RoaringTreemap {
        let get_related = |map: &S::DimensionMap, seg: &Segment| -> RoaringTreemap {
            let mut bitmap = RoaringTreemap::new();
            let mut current = seg.parent();
            while let Some(parent) = current {
                if let Some(ranks) = map.get(&parent) {
                    bitmap |= ranks;
                }
                current = parent.parent();
            }
            let end = seg.descendant_range_end();
            for (_, ranks) in map.range((Included(seg), Excluded(&end))) {
                bitmap |= ranks;
            }
            bitmap
        };

        let f_related = get_related(self.0.f(), target.as_f());
        let x_related = get_related(self.0.x(), target.as_x());
        let y_related = get_related(self.0.y(), target.as_y());
        f_related & x_related & y_related
    }

    fn find_encode(&self, target: &FlexId) -> Option<Rank> {
        let f_hits = self.0.f().get(target.as_f())?;
        let x_hits = self.0.x().get(target.as_x())?;
        let y_hits = self.0.y().get(target.as_y())?;
        (f_hits & x_hits & y_hits).iter().next()
    }

    unsafe fn join_insert_unchecked(&mut self, target: &FlexId, value: &S::V) {
        let try_merge = |logic: &mut Self, sibling: FlexId, parent: FlexId| -> bool {
            if let Some(rank) = logic.find_encode(&sibling) {
                if let Some((_, v)) = logic.0.main().get(&rank) {
                    if v == value {
                        logic.remove_rank(rank);
                        unsafe { logic.join_insert_unchecked(&parent, value) };
                        return true;
                    }
                }
            }
            false
        };

        let f_sibling = FlexId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );
        if let Some(parent_seg) = target.as_f().parent() {
            let parent = FlexId::new(parent_seg, target.as_x().clone(), target.as_y().clone());
            if try_merge(self, f_sibling, parent) {
                return;
            }
        }

        let x_sibling = FlexId::new(
            target.as_f().clone(),
            target.as_x().sibling(),
            target.as_y().clone(),
        );
        if let Some(parent_seg) = target.as_x().parent() {
            let parent = FlexId::new(target.as_f().clone(), parent_seg, target.as_y().clone());
            if try_merge(self, x_sibling, parent) {
                return;
            }
        }

        let y_sibling = FlexId::new(
            target.as_f().clone(),
            target.as_x().clone(),
            target.as_y().sibling(),
        );
        if let Some(parent_seg) = target.as_y().parent() {
            let parent = FlexId::new(target.as_f().clone(), target.as_x().clone(), parent_seg);
            if try_merge(self, y_sibling, parent) {
                return;
            }
        }

        self.insert_unchecked(target, value);
    }

    unsafe fn insert_unchecked(&mut self, target: &FlexId, value: &S::V) {
        let rank = self.0.fetch_next_rank();
        let upsert = |map: &mut S::DimensionMap, key: &Segment| {
            let mut done = false;
            map.update(key, |bm| {
                bm.insert(rank);
                done = true;
            });
            if !done {
                let mut bm = RoaringTreemap::new();
                bm.insert(rank);
                map.insert(key.clone(), bm);
            }
        };

        upsert(self.0.f_mut(), target.as_f());
        upsert(self.0.x_mut(), target.as_x());
        upsert(self.0.y_mut(), target.as_y());

        self.0
            .main_mut()
            .insert(rank, (target.clone(), value.clone()));
    }

    fn remove_rank(&mut self, rank: Rank) -> Option<(FlexId, S::V)> {
        let (encode_id, val) = self.0.main_mut().remove(&rank)?;
        let remove_from_dim = |map: &mut S::DimensionMap, key: &Segment| {
            let mut empty = false;
            map.update(key, |bm| {
                bm.remove(rank);
                if bm.is_empty() {
                    empty = true;
                }
            });
            if empty {
                map.remove(key);
            }
        };
        remove_from_dim(self.0.f_mut(), encode_id.as_f());
        remove_from_dim(self.0.x_mut(), encode_id.as_x());
        remove_from_dim(self.0.y_mut(), encode_id.as_y());
        Some((encode_id, val))
    }
}

// =============================================================================
//  3. Storage: OnMemoryMap
// =============================================================================

#[derive(Clone)]
pub struct OnMemoryMap<V> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<Rank, (FlexId, V)>,
    next_rank: Cell<Rank>,
}

impl<V> OnMemoryMap<V> {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: Cell::new(0),
        }
    }
}

impl<V> Default for OnMemoryMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> MapTrait for OnMemoryMap<V> {
    type V = V;
    type DimensionMap = BTreeMap<Segment, RoaringTreemap>;
    type MainMap = BTreeMap<Rank, (FlexId, V)>;

    fn f(&self) -> &Self::DimensionMap {
        &self.f
    }
    fn f_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.f
    }
    fn x(&self) -> &Self::DimensionMap {
        &self.x
    }
    fn x_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.x
    }
    fn y(&self) -> &Self::DimensionMap {
        &self.y
    }
    fn y_mut(&mut self) -> &mut Self::DimensionMap {
        &mut self.y
    }
    fn main(&self) -> &Self::MainMap {
        &self.main
    }
    fn main_mut(&mut self) -> &mut Self::MainMap {
        &mut self.main
    }
    fn fetch_next_rank(&self) -> Rank {
        let rank = self.next_rank.get();
        self.next_rank.set(rank + 1);
        rank
    }
    fn clear(&mut self) {
        self.f.clear();
        self.x.clear();
        self.y.clear();
        self.main.clear();
        self.next_rank.set(0);
    }
}
