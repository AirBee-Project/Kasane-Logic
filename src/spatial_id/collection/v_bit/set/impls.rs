use crate::{FlexId, FlexIdRankList, RangeId, SingleId, SpatialIdSet, SpatialIds, VBitSet};

impl SpatialIdSet for VBitSet {
    fn insert<T: crate::SpatialId>(&mut self, target: T) {
        let scan_plan = self.core.scan_plan(target);

        //削除が必要なIDを貯めて最後に削除する
        let mut need_remove = FlexIdRankList::new();
        let mut need_insert: Vec<FlexId> = Vec::new();

        for flex_id_scanner in scan_plan.scan() {
            //Parent
            if flex_id_scanner.parent().is_some() {
                continue;
            }

            //Childrent
            need_remove |= flex_id_scanner.children();

            //Partial Overlaps
            let partial_overlaps = flex_id_scanner.partial_overlaps();

            if partial_overlaps.is_empty() {
                need_insert.push(flex_id_scanner.flex_id().clone());
                continue;
            }

            //Setを作成して競合を解消する
            let mut shave_set = Self::new();
            unsafe { shave_set.insert_unchecked(flex_id_scanner.flex_id().clone()) };

            //SetからPartial Overlapsを順番に削除する
            for partial_overlap_rank in partial_overlaps.iter() {
                let flex_id = self.core.find(&partial_overlap_rank).unwrap().0.clone();
                shave_set.remove(flex_id);
            }

            need_insert.extend(shave_set.flex_ids().cloned());
        }

        //削除が必要なものを削除する
        for nend_delete_rank in need_remove.iter() {
            self.core.remove(&nend_delete_rank);
        }

        //挿入するべきものを挿入する
        for need_insert_flex_id in need_insert {
            unsafe { self.join_insert_unchecked(need_insert_flex_id) };
        }
    }

    fn get<T: crate::SpatialId>(&self, target: T) -> Self {
        let scan_plan = self.core.scan_plan(target);
        let mut result = Self::default();

        for flex_id_scanner in scan_plan.scan() {
            //Parent
            if flex_id_scanner.parent().is_some() {
                unsafe { result.join_insert_unchecked(flex_id_scanner.flex_id()) };
                continue;
            }

            //Children
            let _ = flex_id_scanner.children().iter().map(|f| {
                let flex_id = self.core.find(&f).unwrap().0.clone();
                unsafe { result.join_insert_unchecked(flex_id) };
            });

            //Partial Overlap
            let _ = flex_id_scanner.partial_overlaps().iter().map(|f| {
                let flex_id = self.core.find(&f).unwrap().0.clone();
                let intersection = flex_id.intersection(&flex_id_scanner.flex_id()).unwrap();
                unsafe { result.join_insert_unchecked(intersection) };
            });
        }
        result
    }

    fn remove<T: crate::SpatialId>(&mut self, target: T) -> Self {
        let scan_plan = self.core.scan_plan(target);
        let mut result = Self::default();

        //削除すべきFlexIdRank
        let mut need_remove = Vec::new();
        //挿入すべきFlexId
        let mut need_insert = Vec::new();

        for flex_id_scanner in scan_plan.scan() {
            //Parent
            if flex_id_scanner.parent().is_some() {
                unsafe { result.join_insert_unchecked(flex_id_scanner.flex_id()) };
                continue;
            }

            //Children
            let _ = flex_id_scanner.children().iter().map(|f| {
                let flex_id = self.core.find(&f).unwrap().0.clone();
                need_remove.push(f);
                unsafe { result.join_insert_unchecked(flex_id) };
            });

            //Partial Overlap
            let _ = flex_id_scanner.partial_overlaps().iter().map(|f| {
                need_remove.push(f);
                let partial_overlap = self.core.find(&f).unwrap().0.clone();
                for flex_id in partial_overlap.difference(&flex_id_scanner.flex_id()) {
                    need_insert.push(flex_id);
                }
            });
        }

        //need_removeを削除する
        for flex_id_rank in need_remove {
            self.core.remove(&flex_id_rank);
        }

        //need_insertを挿入する
        for flex_id in need_insert {
            self.core.insert(flex_id, ());
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
