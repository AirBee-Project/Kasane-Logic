pub mod tests;

use crate::{
    FlexId, FlexIdRank, RangeId, Segment, SingleId,
    spatial_id::{
        Block, FlexIds,
        collection::{RECYCLE_RANK_MAX, ValueRank, core::SpatialCore, scanner::Scanner},
    },
};
use roaring::RoaringTreemap;
use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    hash::Hash,
};

pub struct TableOnMemory<V: Ord> {
    // ValueRankをCore内で一緒に管理する (HashMap<Rank, (FlexId, ValueRank)>)
    core: SpatialCore<ValueRank>,

    //Table特有の要素
    ///Value側の[RoaringTreemap]にはこのValueを持つ[FlexIdRank]が入っている
    dictionary: BTreeMap<V, (RoaringTreemap, ValueRank)>,
    reverse: HashMap<ValueRank, V>,

    // Value用のRank管理
    value_next_rank: u64,
    value_recycle_rank: Vec<u64>,
}

impl<V> Scanner for TableOnMemory<V>
where
    V: Ord + Clone,
{
    fn f(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        self.core.f()
    }

    fn x(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        self.core.x()
    }

    fn y(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        self.core.y()
    }
}

impl<V> TableOnMemory<V>
where
    V: Ord + Clone + Hash,
{
    ///初期化する
    pub fn new() -> Self {
        Self {
            core: SpatialCore::new(),
            dictionary: BTreeMap::new(),
            reverse: HashMap::new(),
            value_next_rank: 0,
            value_recycle_rank: Vec::new(),
        }
    }

    ///値を挿入する
    pub fn insert<T: Block>(&mut self, target: &T, value: &V) {
        let scanner = self.flex_id_scan_plan(target.clone());
        let mut need_delete_ranks = RoaringTreemap::new();
        let mut need_insert: HashMap<V, Vec<FlexId>> = HashMap::new();

        let exist_same_value = self.find_value(value);

        for flex_id_scanner in scanner.scan() {
            if let Some(parent_rank) = flex_id_scanner.parent() {
                let (parent_flex_id, parent_value_rank) =
                    self.core.get_entry(&parent_rank).unwrap();

                if let Some(value_rank) = exist_same_value {
                    if *parent_value_rank == value_rank {
                        continue;
                    }
                }

                let parent_splited = parent_flex_id.difference(&flex_id_scanner.flex_id());
                let parent_value = self.reverse.get(parent_value_rank).unwrap();

                for splited in parent_splited {
                    need_insert
                        .entry(parent_value.clone())
                        .or_default()
                        .push(splited);
                }
                need_delete_ranks.insert(parent_rank);

                need_insert
                    .entry(value.clone())
                    .or_default()
                    .push(flex_id_scanner.flex_id().clone());

                continue;
            }

            need_delete_ranks |= flex_id_scanner.children();
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            if partial_overlaps.is_empty() {
                need_insert
                    .entry(value.clone())
                    .or_default()
                    .push(flex_id_scanner.flex_id().clone());
                continue;
            }

            need_delete_ranks |= flex_id_scanner.partial_overlaps();

            for partial_overlap_rank in partial_overlaps {
                let (partial_overlap_flex_id, overlap_val_rank) =
                    self.core.get_entry(&partial_overlap_rank).unwrap();
                let overlap_splited =
                    partial_overlap_flex_id.difference(&flex_id_scanner.flex_id());

                let overlap_value = self.reverse.get(overlap_val_rank).unwrap().clone();

                for splited in overlap_splited {
                    need_insert
                        .entry(overlap_value.clone())
                        .or_default()
                        .push(splited);
                }
            }

            need_insert
                .entry(value.clone())
                .or_default()
                .push(flex_id_scanner.flex_id().clone());
        }

        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }

        for (value, flex_ids) in need_insert {
            for flex_id in flex_ids {
                unsafe { self.join_insert_unchecked(flex_id, &value) };
            }
        }
    }

    pub fn range_ids(&self) -> impl Iterator<Item = (RangeId, &V)> {
        self.core.iter().map(|(_, (flex_id, val_rank))| {
            let range_id = flex_id.range_id();
            let val = self.reverse.get(val_rank).unwrap();
            (range_id, val)
        })
    }

    pub fn single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> {
        self.range_ids().flat_map(|(range_id, val)| {
            range_id
                .single_ids()
                .collect::<Vec<SingleId>>()
                .into_iter()
                .map(move |single_id| (single_id, val))
        })
    }

    pub fn get<T: Block>(&self, target: &T) -> Self {
        let scanner = self.flex_id_scan_plan(target.clone());
        let mut result = Self::new();
        for flex_id_scanner in scanner.scan() {
            if let Some(parent_rank) = flex_id_scanner.parent() {
                let parent_value_rank = self.core.get_entry(&parent_rank).unwrap().1;
                let parent_rank = self.reverse.get(&parent_value_rank).unwrap();
                unsafe { result.join_insert_unchecked(flex_id_scanner.flex_id(), parent_rank) };
                continue;
            }

            for child_rank in flex_id_scanner.children() {
                let child = self.core.get_entry(&child_rank).unwrap();
                let child_value = self.reverse.get(&child.1).unwrap();
                unsafe { result.join_insert_unchecked(child.0.clone(), child_value) };
            }

            for partial_overlap_rank in flex_id_scanner.partial_overlaps() {
                let overlap = self.core.get_entry(&partial_overlap_rank).unwrap();
                let overlap_value = self.reverse.get(&overlap.1).unwrap();
                let intersection = overlap.0.intersection(&flex_id_scanner.flex_id()).unwrap();
                unsafe { result.join_insert_unchecked(intersection, overlap_value) };
            }
        }
        result
    }

    pub fn remove<T: Block>(&mut self, target: &T) {
        let scanner = self.flex_id_scan_plan(target.clone());

        let mut need_delete_ranks = RoaringTreemap::new();
        let mut need_insert: HashMap<V, Vec<FlexId>> = HashMap::new();

        for flex_id_scanner in scanner.scan() {
            if let Some(parent_rank) = flex_id_scanner.parent() {
                let (parent_flex_id, parent_value_rank) =
                    self.core.get_entry(&parent_rank).unwrap();

                need_delete_ranks.insert(parent_rank);

                let parent_splited = parent_flex_id.difference(&flex_id_scanner.flex_id());
                let parent_value = self.reverse.get(parent_value_rank).unwrap();

                for splited in parent_splited {
                    need_insert
                        .entry(parent_value.clone())
                        .or_default()
                        .push(splited);
                }

                continue;
            }

            need_delete_ranks |= flex_id_scanner.children();

            let partial_overlaps = flex_id_scanner.partial_overlaps();

            if !partial_overlaps.is_empty() {
                need_delete_ranks |= partial_overlaps.clone();

                for partial_overlap_rank in partial_overlaps {
                    let (partial_overlap_flex_id, overlap_val_rank) =
                        self.core.get_entry(&partial_overlap_rank).unwrap();

                    // 相手 - 自分 = 残すべき部分
                    let overlap_splited =
                        partial_overlap_flex_id.difference(&flex_id_scanner.flex_id());

                    let overlap_value = self.reverse.get(overlap_val_rank).unwrap().clone();

                    for splited in overlap_splited {
                        need_insert
                            .entry(overlap_value.clone())
                            .or_default()
                            .push(splited);
                    }
                }
            }
        }

        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }

        for (value, flex_ids) in need_insert {
            for flex_id in flex_ids {
                unsafe { self.join_insert_unchecked(flex_id, &value) };
            }
        }
    }

    pub unsafe fn join_insert_unchecked<T: Block>(&mut self, target: T, value: &V) {
        let target_val_rank_opt = self.find_value(value);

        for flex_id in target.flex_ids() {
            let check_and_join = |this: &mut Self,
                                  sibling: FlexId,
                                  get_parent: fn(&FlexId) -> Option<FlexId>|
             -> bool {
                let target_val_rank = match target_val_rank_opt {
                    Some(r) => r,
                    None => return false,
                };

                if let Some(contained_ranks) = this.collect_contained_ranks(&sibling) {
                    let all_same_value = contained_ranks.iter().all(|rank| {
                        if let Some((_, v_rank)) = this.core.get_entry(rank) {
                            *v_rank == target_val_rank
                        } else {
                            false
                        }
                    });

                    if !all_same_value {
                        return false;
                    }

                    let total_volume: u128 = contained_ranks
                        .iter()
                        .map(|rank| {
                            let id = this.core.get_flex_id(rank).unwrap();
                            id.volume()
                        })
                        .sum();

                    if total_volume == sibling.volume() {
                        if let Some(parent) = get_parent(&flex_id) {
                            for rank in contained_ranks {
                                this.remove_from_rank(rank);
                            }
                            unsafe { this.join_insert_unchecked(parent, value) };
                            return true;
                        }
                    }
                }
                false
            };

            // F方向の結合チェック
            if check_and_join(self, flex_id.f_sibling(), |id| id.f_parent()) {
                continue;
            }
            // X方向の結合チェック
            if check_and_join(self, flex_id.x_sibling(), |id| id.x_parent()) {
                continue;
            }
            // Y方向の結合チェック
            if check_and_join(self, flex_id.y_sibling(), |id| id.y_parent()) {
                continue;
            }

            // 結合できなければそのまま挿入
            unsafe { self.insert_unchecked(flex_id, value) };
        }
    }

    /// 指定された FlexId の領域内に完全に含まれる（または一致する）全てのランクを取得する。
    fn collect_contained_ranks(&self, target: &FlexId) -> Option<Vec<FlexIdRank>> {
        let f_end = target.as_f().descendant_range_end()?;
        let f_candidates = self.union_bitmaps(self.core.f(), target.as_f(), &f_end);

        if f_candidates.is_empty() {
            return None;
        }

        let x_end = target.as_x().descendant_range_end()?;
        let x_candidates = self.union_bitmaps(self.core.x(), target.as_x(), &x_end);
        if x_candidates.is_empty() {
            return None;
        }

        let fx_intersection = f_candidates & x_candidates;
        if fx_intersection.is_empty() {
            return None;
        }

        let y_end = target.as_y().descendant_range_end()?;
        let y_candidates = self.union_bitmaps(self.core.y(), target.as_y(), &y_end);

        let intersection = fx_intersection & y_candidates;

        if intersection.is_empty() {
            None
        } else {
            Some(intersection.into_iter().collect())
        }
    }

    fn union_bitmaps(
        &self,
        map: &BTreeMap<Segment, RoaringTreemap>,
        start: &Segment,
        end: &Segment,
    ) -> RoaringTreemap {
        let mut result = RoaringTreemap::new();
        for (_, bitmap) in map.range(start..end) {
            result |= bitmap;
        }
        result
    }

    pub unsafe fn insert_unchecked<T: Block>(&mut self, target: T, value: &V) {
        let value_rank = match self.find_value(value) {
            Some(rank) => rank,
            None => self.fetch_value_rank(),
        };

        let mut flex_id_ranks = RoaringTreemap::new();

        for flex_id in target.flex_ids() {
            let rank = self.core.insert_entry(flex_id, value_rank);
            flex_id_ranks.insert(rank);
        }

        match self.dictionary.entry(value.clone()) {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert((flex_id_ranks, value_rank));
                self.reverse.insert(value_rank, value.clone());
            }
            Entry::Occupied(mut occupied_entry) => {
                let exist = occupied_entry.get_mut();
                exist.0 |= flex_id_ranks
            }
        }
    }

    fn find_value(&self, value: &V) -> Option<ValueRank> {
        let value_rank = self.dictionary.get(&value)?.1;
        Some(value_rank)
    }

    fn remove_from_rank(&mut self, rank: FlexIdRank) -> (FlexId, V) {
        let (flex_id, value_rank) = self.core.remove_entry(rank).unwrap();

        let value = self.reverse.get(&value_rank).unwrap().clone();

        let dictionary = self.dictionary.get_mut(&value).unwrap();
        dictionary.0.remove(rank);

        if dictionary.0.is_empty() {
            self.dictionary.remove(&value);
            self.reverse.remove(&value_rank);
            self.return_value_rank(value_rank);
        }

        return (flex_id, value);
    }

    fn fetch_value_rank(&mut self) -> ValueRank {
        match self.value_recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.value_next_rank;
                self.value_next_rank = self.value_next_rank + 1;
                result
            }
        }
    }

    fn return_value_rank(&mut self, rank: u64) {
        if self.value_recycle_rank.len() < RECYCLE_RANK_MAX {
            self.value_recycle_rank.push(rank);
        }
    }
}
