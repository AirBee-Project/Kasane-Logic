use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    hash::Hash,
};

use roaring::RoaringTreemap;

use crate::{
    FlexId, FlexIdRank, RangeId, Segment,
    spatial_id::{
        FlexIds,
        collection::{RECYCLE_RANK_MAX, ValueRank, core::SpatialCore, scanner::Scanner},
    },
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
    pub fn insert<T: FlexIds>(&mut self, target: &T, value: &V) {
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

    pub unsafe fn join_insert_unchecked<T: FlexIds>(&mut self, target: T, value: &V) {
        match self.find_value(value) {
            Some(target_val_rank) => {
                let target_val_rank = target_val_rank.clone();

                for flex_id in target.flex_ids() {
                    // 共通結合ロジック
                    let check_and_join = |this: &mut Self,
                                          neighbor_rank: Option<FlexIdRank>,
                                          get_parent: fn(&FlexId) -> Option<FlexId>|
                     -> bool {
                        if let Some(v) = neighbor_rank {
                            if let Some((_, sibling_v_rank)) = this.core.get_entry(&v) {
                                if *sibling_v_rank == target_val_rank {
                                    if let Some(parent) = get_parent(&flex_id) {
                                        this.remove_from_rank(v);
                                        unsafe { this.join_insert_unchecked(parent, value) };
                                        return true;
                                    }
                                }
                            }
                        }
                        false
                    };

                    // F, X, Y 方向チェック
                    if check_and_join(self, self.core.find(flex_id.f_sibling()), |id| {
                        id.f_parent()
                    }) {
                        continue;
                    }
                    if check_and_join(self, self.core.find(flex_id.x_sibling()), |id| {
                        id.x_parent()
                    }) {
                        continue;
                    }
                    if check_and_join(self, self.core.find(flex_id.y_sibling()), |id| {
                        id.y_parent()
                    }) {
                        continue;
                    }

                    unsafe { self.insert_unchecked(flex_id, value) };
                }
            }
            None => {
                unsafe { self.insert_unchecked(target.clone(), value) };
            }
        }
    }

    pub unsafe fn insert_unchecked<T: FlexIds>(&mut self, target: T, value: &V) {
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
            // 【バグ修正済み】
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
