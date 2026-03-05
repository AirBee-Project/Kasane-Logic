use roaring::RoaringTreemap;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    FlexId, FlexIdRank, RangeId, Segment, SingleId,
    spatial_id::{
        Block, FlexIds,
        collection::{core::SpatialCore, scanner::Scanner},
    },
};

#[derive(Clone, Debug, Default)]
pub struct SetOnMemoryV2 {
    core: SpatialCore<()>,
}

impl Scanner for SetOnMemoryV2 {
    fn f(&self) -> &BTreeMap<Segment<8>, RoaringTreemap> {
        self.core.f()
    }

    fn x(&self) -> &BTreeMap<Segment<8>, RoaringTreemap> {
        self.core.x()
    }

    fn y(&self) -> &BTreeMap<Segment<8>, RoaringTreemap> {
        self.core.y()
    }
}

impl SetOnMemoryV2 {
    pub fn new() -> Self {
        SetOnMemoryV2 {
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

        for need_delete_rank in need_delete_ranks {
            self.remove_from_rank(need_delete_rank);
        }

        for need_insert_flex_id in need_insert {
            unsafe { self.join_insert_unchecked(need_insert_flex_id) };
        }
    }

    /// 断片化への対応を行った結合挿入メソッド（ループ版）。
    /// 単純な兄弟の有無だけでなく、領域内の体積総和を確認して結合判定を行う。
    /// 再帰呼び出しの代わりにループを使用してスタック消費を抑制する。
    pub unsafe fn join_insert_unchecked<T: Block>(&mut self, target: T) {
        for flex_id in target.flex_ids() {
            let mut current = flex_id;
            loop {
                // F方向の結合チェック
                let f_sibling = current.f_sibling();
                if let Some(contained_ranks) = self.collect_contained_ranks(&f_sibling) {
                    let total_volume: u128 = contained_ranks
                        .iter()
                        .map(|rank| self.core.get_flex_id(&rank).unwrap().volume())
                        .sum();
                    if total_volume == f_sibling.volume() {
                        if let Some(parent) = current.f_parent() {
                            let ranks: Vec<u64> = contained_ranks.iter().collect();
                            for rank in ranks {
                                self.remove_from_rank(rank);
                            }
                            current = parent;
                            continue;
                        }
                    }
                }

                // X方向の結合チェック
                let x_sibling = current.x_sibling();
                if let Some(contained_ranks) = self.collect_contained_ranks(&x_sibling) {
                    let total_volume: u128 = contained_ranks
                        .iter()
                        .map(|rank| self.core.get_flex_id(&rank).unwrap().volume())
                        .sum();
                    if total_volume == x_sibling.volume() {
                        if let Some(parent) = current.x_parent() {
                            let ranks: Vec<u64> = contained_ranks.iter().collect();
                            for rank in ranks {
                                self.remove_from_rank(rank);
                            }
                            current = parent;
                            continue;
                        }
                    }
                }

                // Y方向の結合チェック
                let y_sibling = current.y_sibling();
                if let Some(contained_ranks) = self.collect_contained_ranks(&y_sibling) {
                    let total_volume: u128 = contained_ranks
                        .iter()
                        .map(|rank| self.core.get_flex_id(&rank).unwrap().volume())
                        .sum();
                    if total_volume == y_sibling.volume() {
                        if let Some(parent) = current.y_parent() {
                            let ranks: Vec<u64> = contained_ranks.iter().collect();
                            for rank in ranks {
                                self.remove_from_rank(rank);
                            }
                            current = parent;
                            continue;
                        }
                    }
                }

                // 結合できなければそのまま挿入
                self.core.insert_entry(current, ());
                break;
            }
        }
    }

    /// 指定された FlexId の領域内に完全に含まれる（または一致する）全てのランクを
    /// RoaringTreemap のまま返す（Vec への展開を避ける）。
    fn collect_contained_ranks(&self, target: &FlexId) -> Option<RoaringTreemap> {
        let f_end = target.f().descendant_range_end()?;
        let f_candidates = self.union_bitmaps(self.core.f(), target.f(), &f_end);

        if f_candidates.is_empty() {
            return None;
        }

        let x_end = target.x().descendant_range_end()?;
        let x_candidates = self.union_bitmaps(self.core.x(), target.x(), &x_end);
        if x_candidates.is_empty() {
            return None;
        }

        let fx_intersection = f_candidates & x_candidates;
        if fx_intersection.is_empty() {
            return None;
        }

        let y_end = target.y().descendant_range_end()?;
        let y_candidates = self.union_bitmaps(self.core.y(), target.y(), &y_end);

        let intersection = fx_intersection & y_candidates;

        if intersection.is_empty() {
            None
        } else {
            Some(intersection)
        }
    }

    /// BTreeMapの範囲検索を行い、ヒットした全てのRoaringBitmapを結合（Union）して返すヘルパー
    fn union_bitmaps(
        &self,
        map: &BTreeMap<Segment<8>, RoaringTreemap>,
        start: &Segment<8>,
        end: &Segment<8>,
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

    pub fn flex_ids(&self) -> impl Iterator<Item = &FlexId> {
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
            .map(|s: &Segment<8>| s.to_f().0)
            .chain(self.core.x().keys().map(|s: &Segment<8>| s.to_xy().0))
            .chain(self.core.y().keys().map(|s: &Segment<8>| s.to_xy().0))
            .max()
    }

    pub fn min_z(&self) -> Option<u8> {
        self.core
            .f()
            .keys()
            .map(|s: &Segment<8>| s.to_f().0)
            .chain(self.core.x().keys().map(|s: &Segment<8>| s.to_xy().0))
            .chain(self.core.y().keys().map(|s: &Segment<8>| s.to_xy().0))
            .min()
    }

    /// O(n log n) での等値比較。
    /// FlexId を昇順ソートして比較することで、ランク割り当ての違いに依存しない。
    pub fn equal(&self, target: &Self) -> bool {
        if self.size() != target.size() {
            return false;
        }
        let mut a: Vec<&FlexId> = self.flex_ids().collect();
        let mut b: Vec<&FlexId> = target.flex_ids().collect();
        a.sort_unstable();
        b.sort_unstable();
        a == b
    }

    pub fn optimize_single_ids(&self) -> Vec<SingleId> {
        let mut layers: BTreeMap<u8, BTreeSet<SingleId>> = BTreeMap::new();
        for id in self.single_ids() {
            layers.entry(id.z()).or_default().insert(id);
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
                if let Some(parent) = id.spatial_parent(1) {
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
                    .entry((id.z(), id.x(), id.y()))
                    .or_default()
                    .push(id.f());
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

impl FlexIds for SetOnMemoryV2 {
    fn flex_ids(&self) -> impl Iterator<Item = FlexId> {
        self.flex_ids().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::SetOnMemoryV2;
    use crate::{
        F_MAX, F_MIN, MAX_ZOOM_LEVEL, RangeId, SetOnMemory, SingleId, XY_MAX,
        spatial_id::collection::set::tests::{set_a, set_b},
    };
    use proptest::prelude::*;
    use std::collections::HashSet;

    // ── ヘルパー ──────────────────────────────────────────────────────────────

    fn to_flat_set_v2(set: &SetOnMemoryV2, target_z: u8) -> HashSet<SingleId> {
        let mut result = HashSet::new();
        for single_id in set.single_ids() {
            let diff = target_z - single_id.z();
            let children: Vec<_> = single_id.spatial_children(diff).unwrap().collect();
            result.extend(children);
        }
        result
    }

    fn set_a_v2() -> SetOnMemoryV2 {
        let mut set = SetOnMemoryV2::default();
        let id1 = RangeId::new(5, [-7, 11], [1, 5], [5, 30]).unwrap();
        set.insert(&id1);
        let id2 = RangeId::new(3, [2, 2], [1, 5], [2, 2]).unwrap();
        set.insert(&id2);
        set
    }

    fn set_b_v2() -> SetOnMemoryV2 {
        let mut set = SetOnMemoryV2::default();
        let id1 = RangeId::new(4, [5, 4], [4, 5], [9, 10]).unwrap();
        set.insert(&id1);
        let id2 = SingleId::new(2, 2, 2, 2).unwrap();
        set.insert(&id2);
        set
    }

    fn set_c_v2() -> SetOnMemoryV2 {
        let mut set = SetOnMemoryV2::default();
        let id1 = SingleId::new(2, 1, 1, 1).unwrap();
        set.insert(&id1);
        let id2 = SingleId::new(3, 4, 4, 4).unwrap();
        set.insert(&id2);
        let id3 = RangeId::new(4, [-7, 11], [4, 10], [1, 9]).unwrap();
        set.insert(&id3);
        set
    }

    // ── insert テスト ─────────────────────────────────────────────────────────

    #[test]
    fn first_insert_single_id() {
        let mut set = SetOnMemoryV2::new();
        let single_id = SingleId::new(3, 3, 3, 3).unwrap();
        set.insert(&single_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id), range_ids[0]);
    }

    #[test]
    fn first_insert_range_id() {
        let mut set = SetOnMemoryV2::new();
        let range_id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
        set.insert(&range_id);

        let mut single_ids: Vec<SingleId> = set.single_ids().collect();
        let mut answer: Vec<SingleId> = range_id.single_ids().collect();
        answer.sort();
        single_ids.sort();
        assert_eq!(answer, single_ids);
    }

    #[test]
    fn first_insert_single_id_largest() {
        let mut set = SetOnMemoryV2::new();
        let single_id = SingleId::new(0, 0, 0, 0).unwrap();
        set.insert(&single_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id), range_ids[0]);
    }

    #[test]
    fn first_insert_range_id_largest() {
        let mut set = SetOnMemoryV2::new();
        let range_id = RangeId::new(0, [-1, 0], [0, 0], [0, 0]).unwrap();
        set.insert(&range_id);

        let mut single_ids: Vec<SingleId> = set.single_ids().collect();
        let mut answer: Vec<SingleId> = range_id.single_ids().collect();
        answer.sort();
        single_ids.sort();
        assert_eq!(answer, single_ids);
    }

    #[test]
    fn first_insert_single_id_smallest() {
        let mut set = SetOnMemoryV2::new();
        let single_id = SingleId::new(MAX_ZOOM_LEVEL as u8, 10, 10, 10).unwrap();
        set.insert(&single_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id), range_ids[0]);
    }

    #[test]
    fn first_insert_single_id_smallest_edge_start() {
        let mut set = SetOnMemoryV2::new();
        let single_id =
            SingleId::new(MAX_ZOOM_LEVEL as u8, F_MIN[MAX_ZOOM_LEVEL], 0, 0).unwrap();
        set.insert(&single_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id), range_ids[0]);
    }

    #[test]
    fn first_insert_single_id_smallest_edge_end() {
        let mut set = SetOnMemoryV2::new();
        let single_id = SingleId::new(
            MAX_ZOOM_LEVEL as u8,
            F_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
            XY_MAX[MAX_ZOOM_LEVEL],
        )
        .unwrap();
        set.insert(&single_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id), range_ids[0]);
    }

    #[test]
    fn multiple_insert_single_id_overlap() {
        let mut set = SetOnMemoryV2::new();
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(3, 1, 1, 0).unwrap();
        set.insert(&single_id_a);
        set.insert(&single_id_b);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(RangeId::from(single_id_b), range_ids[0]);
    }

    #[test]
    fn multiple_insert_single_id_join() {
        let mut set = SetOnMemoryV2::new();
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 0).unwrap();
        set.insert(&single_id_a);
        set.insert(&single_id_b);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(range_ids[0], RangeId::new(4, [3, 3], [2, 2], [0, 1]).unwrap());
    }

    #[test]
    fn multiple_insert_single_id_no_join() {
        let mut set = SetOnMemoryV2::new();
        let single_id_a = SingleId::new(4, 3, 2, 1).unwrap();
        let single_id_b = SingleId::new(4, 3, 2, 2).unwrap();
        set.insert(&single_id_a);
        set.insert(&single_id_b);

        let mut range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(2, range_ids.len());

        let mut answer = vec![RangeId::from(single_id_a), RangeId::from(single_id_b)];
        range_ids.sort();
        answer.sort();
        assert_eq!(range_ids, answer);
    }

    #[test]
    fn first_insert_range_id_join() {
        let mut set = SetOnMemoryV2::new();
        let range_id = RangeId::new(4, [0, F_MAX[4]], [0, XY_MAX[4]], [0, XY_MAX[4]]).unwrap();
        set.insert(&range_id);

        let range_ids: Vec<RangeId> = set.range_ids().collect();
        assert_eq!(1, range_ids.len());
        assert_eq!(range_ids[0], RangeId::new(0, [0, 0], [0, 0], [0, 0]).unwrap());
    }

    // ── union テスト ──────────────────────────────────────────────────────────

    fn assert_union_v2(result: &SetOnMemoryV2, inputs: &[&SetOnMemoryV2], msg: &str) {
        let max_z = inputs
            .iter()
            .map(|s| s.max_z())
            .chain(std::iter::once(result.max_z()))
            .max()
            .unwrap()
            .unwrap_or(0);

        let actual = to_flat_set_v2(result, max_z);
        let mut expected = HashSet::new();
        for s in inputs {
            expected.extend(to_flat_set_v2(s, max_z));
        }
        assert_eq!(actual, expected, "{}", msg);
    }

    #[test]
    fn test_union() {
        let a = set_a_v2();
        let b = set_b_v2();
        let result = a.union(&b);
        assert_union_v2(&result, &[&a, &b], "union(A, B) failed");
    }

    #[test]
    fn test_union_three_sets() {
        let a = set_a_v2();
        let b = set_b_v2();
        let c = set_c_v2();
        let result = a.union(&b).union(&c);
        assert_union_v2(&result, &[&a, &b, &c], "union(A, B, C) failed");
    }

    // ── intersection テスト ───────────────────────────────────────────────────

    fn assert_intersection_v2(result: &SetOnMemoryV2, inputs: &[&SetOnMemoryV2], msg: &str) {
        if inputs.is_empty() {
            return;
        }
        let max_z = inputs
            .iter()
            .map(|s| s.max_z())
            .chain(std::iter::once(result.max_z()))
            .max()
            .unwrap()
            .unwrap_or(0);

        let actual = to_flat_set_v2(result, max_z);
        let mut expected = to_flat_set_v2(inputs[0], max_z);
        for s in &inputs[1..] {
            let other = to_flat_set_v2(s, max_z);
            expected.retain(|id| other.contains(id));
        }
        assert_eq!(actual, expected, "{}", msg);
    }

    #[test]
    fn test_intersection() {
        let a = set_a_v2();
        let b = set_b_v2();
        let result = a.intersection(&b);
        assert_intersection_v2(&result, &[&a, &b], "intersection(A, B) failed");
    }

    #[test]
    fn test_intersection_three_sets() {
        let a = set_a_v2();
        let b = set_b_v2();
        let c = set_c_v2();
        let result = a.intersection(&b).intersection(&c);
        assert_intersection_v2(&result, &[&a, &b, &c], "intersection(A, B, C) failed");
    }

    // ── difference テスト ─────────────────────────────────────────────────────

    fn assert_difference_v2(
        result: &SetOnMemoryV2,
        initial: &SetOnMemoryV2,
        subtractors: &[&SetOnMemoryV2],
        msg: &str,
    ) {
        let max_z = std::iter::once(result.max_z())
            .chain(std::iter::once(initial.max_z()))
            .chain(subtractors.iter().map(|s| s.max_z()))
            .max()
            .unwrap()
            .unwrap_or(0);

        let actual = to_flat_set_v2(result, max_z);
        let mut expected = to_flat_set_v2(initial, max_z);
        for s in subtractors {
            let sub = to_flat_set_v2(s, max_z);
            expected.retain(|id| !sub.contains(id));
        }
        assert_eq!(actual, expected, "{}", msg);
    }

    #[test]
    fn test_difference() {
        let a = set_a_v2();
        let b = set_b_v2();
        let result = a.difference(&b);
        assert_difference_v2(&result, &a, &[&b], "difference(A, B) failed");
    }

    #[test]
    fn test_difference_three_sets() {
        let a = set_a_v2();
        let b = set_b_v2();
        let c = set_c_v2();
        let result = a.difference(&b).difference(&c);
        assert_difference_v2(&result, &a, &[&b, &c], "difference(A, B, C) failed");
    }

    // ── equal テスト ──────────────────────────────────────────────────────────

    #[test]
    fn test_equal_same_set() {
        let a = set_a_v2();
        assert!(a.equal(&a.clone()), "A should equal itself");
    }

    #[test]
    fn test_equal_different_sets() {
        let a = set_a_v2();
        let b = set_b_v2();
        assert!(!a.equal(&b), "A should not equal B");
    }

    #[test]
    fn test_equal_after_union() {
        let a = set_a_v2();
        let b = set_b_v2();
        let ab1 = a.union(&b);
        let ab2 = b.union(&a);
        assert!(ab1.equal(&ab2), "A∪B should equal B∪A");
    }

    #[test]
    fn test_equal_empty() {
        let a = SetOnMemoryV2::new();
        let b = SetOnMemoryV2::new();
        assert!(a.equal(&b), "two empty sets should be equal");
    }

    // ── SetOnMemory vs SetOnMemoryV2 パリティテスト ─────────────────────────

    fn v1_single_ids_sorted(set: &SetOnMemory) -> Vec<SingleId> {
        let mut ids: Vec<SingleId> = set.single_ids().collect();
        ids.sort();
        ids
    }

    fn v2_single_ids_sorted(set: &SetOnMemoryV2) -> Vec<SingleId> {
        let mut ids: Vec<SingleId> = set.single_ids().collect();
        ids.sort();
        ids
    }

    #[test]
    fn parity_insert_single_id() {
        let id = SingleId::new(4, 3, 2, 1).unwrap();
        let mut v1 = SetOnMemory::new();
        v1.insert(&id);
        let mut v2 = SetOnMemoryV2::new();
        v2.insert(&id);
        assert_eq!(v1_single_ids_sorted(&v1), v2_single_ids_sorted(&v2));
    }

    #[test]
    fn parity_insert_range_id() {
        let id = RangeId::new(4, [-4, 5], [2, 10], [3, 3]).unwrap();
        let mut v1 = SetOnMemory::new();
        v1.insert(&id);
        let mut v2 = SetOnMemoryV2::new();
        v2.insert(&id);
        assert_eq!(v1_single_ids_sorted(&v1), v2_single_ids_sorted(&v2));
    }

    #[test]
    fn parity_union() {
        let (a1, b1) = (set_a(), set_b());
        let (a2, b2) = (set_a_v2(), set_b_v2());
        let r1 = a1.union(&b1);
        let r2 = a2.union(&b2);
        assert_eq!(v1_single_ids_sorted(&r1), v2_single_ids_sorted(&r2));
    }

    #[test]
    fn parity_intersection() {
        let (a1, b1) = (set_a(), set_b());
        let (a2, b2) = (set_a_v2(), set_b_v2());
        let r1 = a1.intersection(&b1);
        let r2 = a2.intersection(&b2);
        assert_eq!(v1_single_ids_sorted(&r1), v2_single_ids_sorted(&r2));
    }

    #[test]
    fn parity_difference() {
        let (a1, b1) = (set_a(), set_b());
        let (a2, b2) = (set_a_v2(), set_b_v2());
        let r1 = a1.difference(&b1);
        let r2 = a2.difference(&b2);
        assert_eq!(v1_single_ids_sorted(&r1), v2_single_ids_sorted(&r2));
    }

    /// Generates a pair of (SetOnMemory, SetOnMemoryV2) built from the same random insert
    /// sequence. This ensures they have identical internal state (same ranks), so that
    /// subsequent operations produce identical results.
    fn arb_small_set_pair(
        max_len: usize,
    ) -> impl proptest::strategy::Strategy<Value = (SetOnMemory, SetOnMemoryV2)> {
        use proptest::prop_oneof;
        let z_range = 0u8..=4;

        #[derive(Debug, Clone)]
        enum Elem {
            Single(SingleId),
            Range(RangeId),
        }

        let elem_strategy = prop_oneof![
            SingleId::arb_within(z_range.clone()).prop_map(Elem::Single),
            RangeId::arb_within(z_range).prop_map(Elem::Range),
        ];

        proptest::collection::vec(elem_strategy, 0..=max_len).prop_map(|elems| {
            let mut v1 = SetOnMemory::default();
            let mut v2 = SetOnMemoryV2::default();
            for elem in elems {
                match elem {
                    Elem::Single(id) => {
                        v1.insert(&id);
                        v2.insert(&id);
                    }
                    Elem::Range(id) => {
                        v1.insert(&id);
                        v2.insert(&id);
                    }
                }
            }
            (v1, v2)
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn random_parity_insert((set_v1, set_v2) in arb_small_set_pair(20)) {
            assert_eq!(
                v1_single_ids_sorted(&set_v1),
                v2_single_ids_sorted(&set_v2),
                "V1 and V2 produced different single_ids for the same input"
            );
        }

        #[test]
        fn random_parity_union(
            (set_a, a2) in arb_small_set_pair(15),
            (set_b, b2) in arb_small_set_pair(15),
        ) {
            let r1 = set_a.union(&set_b);
            let r2 = a2.union(&b2);

            assert_eq!(v1_single_ids_sorted(&r1), v2_single_ids_sorted(&r2),
                "union parity failed");
        }

        #[test]
        fn random_parity_difference(
            (set_a, a2) in arb_small_set_pair(15),
            (set_b, b2) in arb_small_set_pair(15),
        ) {
            let r1 = set_a.difference(&set_b);
            let r2 = a2.difference(&b2);

            assert_eq!(v1_single_ids_sorted(&r1), v2_single_ids_sorted(&r2),
                "difference parity failed");
        }
    }
}
