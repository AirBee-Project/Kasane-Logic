use roaring::RoaringTreemap;
use std::collections::{BTreeMap, BTreeSet};

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

    /// 断片化への対応を行った結合挿入メソッド。
    /// 単純な兄弟の有無だけでなく、領域内の体積総和を確認して結合判定を行う。
    pub unsafe fn join_insert_unchecked<T: Block>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            // 結合チェック用クロージャ
            let check_and_join = |this: &mut Self,
                                  sibling: FlexId,
                                  get_parent: fn(&FlexId) -> Option<FlexId>|
             -> bool {
                if let Some(contained_ranks) = this.collect_contained_ranks(&sibling) {
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
                            unsafe { this.join_insert_unchecked(parent) };
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
            unsafe { self.insert_unchecked(flex_id) };
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

    /// BTreeMapの範囲検索を行い、ヒットした全てのRoaringBitmapを結合（Union）して返すヘルパー
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

    pub unsafe fn insert_unchecked<T: Block>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            self.core.insert_entry(flex_id, ());
        }
    }

    pub fn remove<T: Block>(&mut self, target: &T) {
        let scanner = self.flex_id_scan_plan(target.clone());

        let mut need_delete_ranks: Vec<FlexIdRank> = Vec::new();
        let mut need_insert_flex_ids: Vec<FlexId> = Vec::new();

        for scan_result in scanner.scan() {
            if let Some(parent_rank) = scan_result.parent() {
                if let Some(parent_flex_id) = self.core.get_flex_id(&parent_rank) {
                    let diff = parent_flex_id.difference(&scan_result.flex_id());
                    need_delete_ranks.push(parent_rank);
                    need_insert_flex_ids.extend(diff);
                }
            } else {
                let children_ranks = scan_result.children();
                need_delete_ranks.extend(children_ranks);
                for partial_overlap_rank in scan_result.partial_overlaps() {
                    if let Some(base_flex_id) = self.core.get_flex_id(&partial_overlap_rank) {
                        let diff = base_flex_id.difference(&scan_result.flex_id());
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

    pub fn optimize_single_ids(&self) -> Vec<SingleId> {
        let mut layers: BTreeMap<u8, BTreeSet<SingleId>> = BTreeMap::new();
        for id in self.single_ids() {
            layers.entry(id.as_z()).or_default().insert(id);
        }
        if layers.is_empty() {
            return Vec::new();
        }
        let max_z = *layers.keys().next_back().unwrap();
        for z in (1..=max_z).rev() {
            let ids_at_current_z = match layers.remove(&z) {
                Some(ids) => ids,
                None => continue,
            };
            let mut siblings_map: BTreeMap<SingleId, Vec<SingleId>> = BTreeMap::new();
            for id in ids_at_current_z {
                if let Some(parent) = id.parent(1) {
                    siblings_map.entry(parent).or_default().push(id);
                }
            }
            for (parent, children) in siblings_map {
                if children.len() == 8 {
                    layers.entry(z - 1).or_default().insert(parent);
                } else {
                    let layer = layers.entry(z).or_default();
                    for child in children {
                        layer.insert(child);
                    }
                }
            }
        }
        layers.into_values().flatten().collect()
    }

    pub fn optimize_range_ids(&self) -> Vec<RangeId> {
        let singles = self.optimize_single_ids();
        if singles.is_empty() {
            return Vec::new();
        }
        let mut x_map: BTreeMap<(u8, u32, [i32; 2]), Vec<u32>> = BTreeMap::new();
        {
            let mut f_map: BTreeMap<(u8, u32, u32), Vec<i32>> = BTreeMap::new();
            for id in singles {
                f_map
                    .entry((id.as_z(), id.as_x(), id.as_y()))
                    .or_default()
                    .push(id.as_f());
            }
            for ((z, x, y), mut fs) in f_map {
                fs.sort_unstable();
                for f_range in Self::merge_indices(&fs) {
                    x_map.entry((z, y, f_range)).or_default().push(x);
                }
            }
        }

        let mut y_map: BTreeMap<(u8, [i32; 2], [u32; 2]), Vec<u32>> = BTreeMap::new();
        for ((z, y, f_range), mut xs) in x_map {
            xs.sort_unstable();
            let xy_max = crate::spatial_id::constants::XY_MAX[z as usize];
            // X方向のみ循環を考慮
            for x_range in Self::merge_indices_with_wrap(&xs, xy_max) {
                y_map.entry((z, f_range, x_range)).or_default().push(y);
            }
        }

        let mut results = Vec::new();
        for ((z, f_range, x_range), mut ys) in y_map {
            ys.sort_unstable();
            for y_range in Self::merge_indices(&ys) {
                if let Ok(id) = RangeId::new(z, f_range, x_range, y_range) {
                    results.push(id);
                }
            }
        }
        results
    }

    fn merge_indices<T>(indices: &[T]) -> Vec<[T; 2]>
    where
        T: Copy + PartialEq + std::ops::Add<Output = T> + From<u8>,
    {
        if indices.is_empty() {
            return Vec::new();
        }
        let mut ranges = Vec::new();
        let mut start = indices[0];
        let mut prev = indices[0];

        for &curr in &indices[1..] {
            if curr != prev + T::from(1) {
                ranges.push([start, prev]);
                start = curr;
            }
            prev = curr;
        }
        ranges.push([start, prev]);
        ranges
    }

    fn merge_indices_with_wrap(indices: &[u32], max_val: u32) -> Vec<[u32; 2]> {
        let mut ranges = Self::merge_indices(indices);
        if ranges.len() > 1 && ranges[0][0] == 0 && ranges.last().unwrap()[1] == max_val {
            let first = ranges.remove(0);
            let last = ranges.pop().unwrap();
            ranges.push([last[0], first[1]]);
        }
        ranges
    }
}

impl FlexIds for SetOnMemory {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId> {
        self.flex_ids().cloned()
    }
}
