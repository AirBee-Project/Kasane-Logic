use crate::{FlexId, RangeId, SingleId, SpatialIdSet, SpatialIds, VBitSet};
use std::collections::HashSet;

impl SpatialIdSet for VBitSet {
    fn insert<T: crate::SpatialId>(&mut self, target: T) {
        let mut affected_ranks = HashSet::new();
        let mut target_flex_ids = Vec::new();

        {
            let scan_plan = self.core.scan_plan(target);
            for scanner in scan_plan.scan() {
                target_flex_ids.push(scanner.flex_id());

                if let Some(p) = scanner.parent() {
                    affected_ranks.insert(p);
                }
                for c in scanner.children().iter() {
                    affected_ranks.insert(c);
                }
                for p in scanner.partial_overlaps().iter() {
                    affected_ranks.insert(p);
                }
            }
        }

        let mut workspace = Self::default();
        for rank in affected_ranks {
            if let Some((flex_id, _)) = self.core.remove(&rank) {
                unsafe { workspace.join_insert_unchecked(flex_id) };
            }
        }

        for target_id in target_flex_ids {
            let mut need_remove = Vec::new();
            let mut need_insert = Vec::new();

            {
                let plan = workspace.core.scan_plan(target_id.clone());
                let scanner = match plan.scan().next() {
                    Some(s) => s,
                    None => continue,
                };

                if scanner.parent().is_some() {
                    continue;
                }
                for c in scanner.children().iter() {
                    need_remove.push(c);
                }
                for p in scanner.partial_overlaps().iter() {
                    need_remove.push(p);
                    let p_id = workspace.core.find(&p).unwrap().0.clone();
                    need_insert.extend(p_id.difference(&target_id));
                }
                need_insert.push(target_id);
            }

            for r in need_remove {
                workspace.core.remove(&r);
            }
            for id in need_insert {
                unsafe { workspace.join_insert_unchecked(id) };
            }
        }

        for flex_id in workspace.flex_ids() {
            unsafe { self.join_insert_unchecked(flex_id.clone()) };
        }
    }

    fn get<T: crate::SpatialId>(&self, target: T) -> Self {
        let scan_plan = self.core.scan_plan(target);
        let mut result = Self::default();

        for scanner in scan_plan.scan() {
            if scanner.parent().is_some() {
                unsafe { result.join_insert_unchecked(scanner.flex_id()) };
                continue;
            }
            for f in scanner.children().iter() {
                let flex_id = self.core.find(&f).unwrap().0.clone();
                unsafe { result.join_insert_unchecked(flex_id) };
            }
            for f in scanner.partial_overlaps().iter() {
                let flex_id = self.core.find(&f).unwrap().0.clone();
                if let Some(intersection) = flex_id.intersection(&scanner.flex_id()) {
                    unsafe { result.join_insert_unchecked(intersection) };
                }
            }
        }
        result
    }

    fn remove<T: crate::SpatialId>(&mut self, target: T) -> Self {
        let mut affected_ranks = HashSet::new();
        let mut target_flex_ids = Vec::new();

        {
            let scan_plan = self.core.scan_plan(target);
            for scanner in scan_plan.scan() {
                target_flex_ids.push(scanner.flex_id());
                if let Some(p) = scanner.parent() {
                    affected_ranks.insert(p);
                }
                for c in scanner.children().iter() {
                    affected_ranks.insert(c);
                }
                for p in scanner.partial_overlaps().iter() {
                    affected_ranks.insert(p);
                }
            }
        }

        let mut workspace = Self::default();
        for rank in affected_ranks {
            if let Some((flex_id, _)) = self.core.remove(&rank) {
                unsafe { workspace.join_insert_unchecked(flex_id) };
            }
        }

        let mut result = Self::default();

        for target_id in target_flex_ids {
            let mut need_remove = Vec::new();
            let mut workspace_insert = Vec::new();
            let mut result_insert = Vec::new();

            {
                let plan = workspace.core.scan_plan(target_id.clone());
                let scanner = match plan.scan().next() {
                    Some(s) => s,
                    None => continue,
                };

                if let Some(parent_rank) = scanner.parent() {
                    need_remove.push(parent_rank);
                    result_insert.push(target_id.clone());
                    let p_id = workspace.core.find(&parent_rank).unwrap().0.clone();
                    workspace_insert.extend(p_id.difference(&target_id));
                } else {
                    for c in scanner.children().iter() {
                        need_remove.push(c);
                        result_insert.push(workspace.core.find(&c).unwrap().0.clone());
                    }
                    for p in scanner.partial_overlaps().iter() {
                        need_remove.push(p);
                        let p_id = workspace.core.find(&p).unwrap().0.clone();
                        if let Some(intersect) = p_id.intersection(&target_id) {
                            result_insert.push(intersect);
                        }
                        workspace_insert.extend(p_id.difference(&target_id));
                    }
                }
            }

            for r in need_remove {
                workspace.core.remove(&r);
            }
            for id in workspace_insert {
                unsafe { workspace.join_insert_unchecked(id) };
            }
            for id in result_insert {
                unsafe { result.join_insert_unchecked(id) };
            }
        }

        for flex_id in workspace.flex_ids() {
            unsafe { self.join_insert_unchecked(flex_id.clone()) };
        }

        result
    }

    fn size(&self) -> usize {
        self.core.size()
    }
    fn clear(&mut self) {
        self.core.clear();
    }
    fn is_empty(&self) -> bool {
        self.core.is_empty()
    }
}

impl SpatialIds for VBitSet {
    type SingleItem<'a> = SingleId;
    type RangeItem<'a> = RangeId;
    type FlexItem<'a> = &'a FlexId;

    fn single_ids(&self) -> impl Iterator<Item = Self::SingleItem<'_>> {
        self.flex_ids().flat_map(|f| f.single_ids())
    }

    fn range_ids(&self) -> impl Iterator<Item = Self::RangeItem<'_>> {
        self.flex_ids().flat_map(|f| f.range_ids())
    }

    fn flex_ids(&self) -> impl Iterator<Item = Self::FlexItem<'_>> {
        self.core.iter().map(|(_, (flex_id, _))| flex_id)
    }
}
