use roaring::RoaringTreemap;
use std::collections::BTreeMap;

pub mod tests;

use crate::{
    FlexId, FlexIdRank, RangeId, Segment, SingleId,
    spatial_id::{
        Block, FlexIds,
        collection::{core::SpatialCore, scanner::Scanner},
    },
};

#[derive(Clone, Debug, Default)]
pub struct SetOnMemory {
    core: SpatialCore<()>,
}

impl Scanner for SetOnMemory {
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

impl SetOnMemory {
    pub fn new() -> Self {
        SetOnMemory {
            core: SpatialCore::new(),
        }
    }

    pub fn insert<T: Block>(&mut self, target: &T) {
        let scanner = self.flex_id_scan_plan(target.clone());
        let mut need_delete_ranks = RoaringTreemap::new();
        let mut need_insert: Vec<FlexId> = Vec::new();

        for flex_id_scanner in scanner.scan() {
            if flex_id_scanner.parent().is_some() {
                continue;
            }

            need_delete_ranks |= flex_id_scanner.children();
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            if partial_overlaps.is_empty() {
                need_insert.push(flex_id_scanner.flex_id().clone());
                continue;
            }

            let mut shave_set = Self::new();
            unsafe { shave_set.insert_unchecked(flex_id_scanner.flex_id().clone()) };

            for partial_overlap_rank in partial_overlaps {
                let flex_id = self.core.get_flex_id(&partial_overlap_rank).unwrap();
                shave_set.remove(flex_id);
            }

            need_insert.extend(shave_set.flex_ids().cloned());
        }

        for nend_delete_rank in need_delete_ranks {
            self.remove_from_rank(nend_delete_rank);
        }

        for need_insert_flex_id in need_insert {
            unsafe { self.join_insert_unchecked(need_insert_flex_id) };
        }
    }

    pub unsafe fn join_insert_unchecked<T: Block>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            let check_and_join = |this: &mut Self,
                                  neighbor_rank: Option<FlexIdRank>,
                                  get_parent: fn(&FlexId) -> Option<FlexId>|
             -> bool {
                if let Some(v) = neighbor_rank {
                    if let Some(parent) = get_parent(&flex_id) {
                        this.remove_from_rank(v);
                        unsafe { this.join_insert_unchecked(parent) };
                        return true;
                    }
                }
                false
            };

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

            unsafe { self.insert_unchecked(flex_id) };
        }
    }

    pub unsafe fn insert_unchecked<T: Block>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            // ダミーの () を渡す
            self.core.insert_entry(flex_id, ());
        }
    }

    pub fn remove<T: Block>(&mut self, target: &T) {
        for flex_id in target.flex_ids() {
            let scanner = self.flex_id_scan_plan(target.clone());

            let mut need_delete_ranks: Vec<FlexIdRank> = Vec::new();
            let mut need_insert_flex_ids: Vec<FlexId> = Vec::new();

            for scan_result in scanner.scan() {
                if let Some(parent_rank) = scan_result.parent() {
                    if let Some(parent_flex_id) = self.core.get_flex_id(&parent_rank) {
                        let diff = parent_flex_id.difference(&flex_id);
                        need_delete_ranks.push(parent_rank);
                        need_insert_flex_ids.extend(diff);
                    }
                } else {
                    let children_ranks = scan_result.children();
                    need_delete_ranks.extend(children_ranks);
                    for partial_overlap_rank in scan_result.partial_overlaps() {
                        if let Some(base_flex_id) = self.core.get_flex_id(&partial_overlap_rank) {
                            let diff = base_flex_id.difference(&flex_id);
                            need_delete_ranks.push(partial_overlap_rank);
                            need_insert_flex_ids.extend(diff);
                        }
                    }
                }
            }
            for rank in need_delete_ranks {
                self.remove_from_rank(rank);
            }
            for insert_id in need_insert_flex_ids {
                unsafe { self.join_insert_unchecked(insert_id) };
            }
        }
    }

    pub fn size(&self) -> usize {
        self.core.len()
    }

    pub fn get<T: Block>(&self, target: &T) -> Self {
        let scanner = self.flex_id_scan_plan(target.clone());
        let mut result = Self::new();
        for flex_id_scanner in scanner.scan() {
            if flex_id_scanner.parent().is_some() {
                unsafe { result.join_insert_unchecked(flex_id_scanner.flex_id()) };
                continue;
            }

            for child_rank in flex_id_scanner.children() {
                let flex_id = self.core.get_flex_id(&child_rank).unwrap();
                unsafe { result.join_insert_unchecked(flex_id.clone()) };
            }

            for partial_overlap_rank in flex_id_scanner.partial_overlaps() {
                let overlap_flex_id = self.core.get_flex_id(&partial_overlap_rank).unwrap();
                let intersection = overlap_flex_id
                    .intersection(&flex_id_scanner.flex_id())
                    .unwrap();
                unsafe { result.join_insert_unchecked(intersection) };
            }
        }
        result
    }

    fn remove_from_rank(&mut self, rank: FlexIdRank) -> FlexId {
        self.core.remove_entry(rank).unwrap().0
    }

    fn flex_ids(&self) -> impl Iterator<Item = &FlexId> {
        self.core.iter().map(|(_, (v, _))| v)
    }

    pub fn join(&mut self, target: &Self) {
        for flex_id in target.flex_ids() {
            self.insert(flex_id);
        }
    }

    pub fn union(&self, target: &Self) -> Self {
        let mut result;
        if self.size() > target.size() {
            result = self.clone();
            for flex_id in target.flex_ids() {
                result.insert(flex_id);
            }
        } else {
            result = target.clone();
            for flex_id in self.flex_ids() {
                result.insert(flex_id);
            }
        }
        result
    }

    pub fn intersection(&self, target: &Self) -> Self {
        let mut result = Self::new();
        if self.size() > target.size() {
            for flex_id in target.flex_ids() {
                let intersect = self.get(flex_id);
                result.join(&intersect);
            }
        } else {
            for flex_id in self.flex_ids() {
                let intersect = target.get(flex_id);
                result.join(&intersect);
            }
        }
        result
    }

    pub fn difference(&self, target: &Self) -> Self {
        let mut result = self.clone();
        for flex_id in target.flex_ids() {
            result.remove(flex_id);
        }
        result
    }

    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> {
        self.core.iter().map(|(_, (flex_id, _))| flex_id.range_id())
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> {
        self.range_ids()
            .flat_map(|f| f.single_ids().collect::<Vec<_>>())
    }

    pub fn is_empty(&self) -> bool {
        self.core.is_empty()
    }

    pub fn max_z(&self) -> Option<u8> {
        self.core
            .f()
            .keys()
            .map(|s| s.to_f().0)
            .chain(self.core.x().keys().map(|s| s.to_xy().0))
            .chain(self.core.y().keys().map(|s| s.to_xy().0))
            .max()
    }

    pub fn min_z(&self) -> Option<u8> {
        self.core
            .f()
            .keys()
            .map(|s| s.to_f().0)
            .chain(self.core.x().keys().map(|s| s.to_xy().0))
            .chain(self.core.y().keys().map(|s| s.to_xy().0))
            .min()
    }

    pub fn equal(&self, target: &Self) -> bool {
        if self.size() != target.size() {
            return false;
        }
        let diff1 = self.difference(target);
        if !diff1.is_empty() {
            return false;
        }
        let diff2 = target.difference(self);
        diff2.is_empty()
    }
}

impl FlexIds for SetOnMemory {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId> {
        self.flex_ids().cloned()
    }
}
