pub mod flex_id_rank;
pub mod flex_id_rank_list;
pub mod scanner;

use std::collections::BTreeMap;

use crate::{FlexId, FlexIdRank, FlexIdRankList, FlexIdScanPlan, Segment, SpatialId};

#[derive(Debug, Clone)]
pub struct VBitCore<T> {
    f: BTreeMap<Segment<8>, FlexIdRankList>,
    x: BTreeMap<Segment<8>, FlexIdRankList>,
    y: BTreeMap<Segment<8>, FlexIdRankList>,
    main: BTreeMap<FlexIdRank, (FlexId, T)>,
}

impl<T> VBitCore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(&self) -> usize {
        self.main.len()
    }

    pub fn is_empty(&self) -> bool {
        self.main.is_empty()
    }

    pub fn clear(&mut self) {
        self.f.clear();
        self.x.clear();
        self.y.clear();
        self.main.clear();
    }

    pub fn scan_plan<S: SpatialId>(&'_ self, target: S) -> FlexIdScanPlan<'_> {
        FlexIdScanPlan::new(&self, target)
    }

    pub fn insert(&mut self, flex_id: FlexId, value: T) {
        let dimension_segment = |btree: &mut BTreeMap<Segment<8>, FlexIdRankList>,
                                 segment: &Segment<8>,
                                 flex_id_rank: FlexIdRank| {
            match btree.entry(segment.clone()) {
                std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                    let mut new_flex_id_rank_list = FlexIdRankList::new();
                    new_flex_id_rank_list.insert(flex_id_rank);
                    vacant_entry.insert(new_flex_id_rank_list);
                }
                std::collections::btree_map::Entry::Occupied(mut occupied_entry) => {
                    let flex_id_rank_list = occupied_entry.get_mut();
                    flex_id_rank_list.insert(flex_id_rank);
                }
            }
        };

        let flex_id_rank = flex_id.flex_id_rank();

        dimension_segment(&mut self.f, flex_id.f_segment(), flex_id_rank);
        dimension_segment(&mut self.x, flex_id.x_segment(), flex_id_rank);
        dimension_segment(&mut self.y, flex_id.y_segment(), flex_id_rank);

        self.main.insert(flex_id_rank, (flex_id, value));
    }

    pub fn contains(&self, flex_id: &FlexId) -> bool {
        self.main.contains_key(&flex_id.flex_id_rank())
    }

    pub fn find(&self, flex_id_rank: &FlexIdRank) -> Option<&(FlexId, T)> {
        self.main.get(&flex_id_rank)
    }

    pub fn remove(&mut self, flex_id_rank: &FlexIdRank) -> Option<(FlexId, T)> {
        let (flex_id, value) = self.main.remove(flex_id_rank)?;

        let remove_from_dim = |btree: &mut BTreeMap<Segment<8>, FlexIdRankList>,
                               segment: &Segment<8>| {
            if let std::collections::btree_map::Entry::Occupied(mut entry) =
                btree.entry(segment.clone())
            {
                let list = entry.get_mut();
                list.remove(flex_id_rank.clone());
                if list.is_empty() {
                    entry.remove_entry();
                }
            }
        };

        remove_from_dim(&mut self.f, flex_id.f_segment());
        remove_from_dim(&mut self.x, flex_id.x_segment());
        remove_from_dim(&mut self.y, flex_id.y_segment());

        Some((flex_id, value))
    }
    pub fn f(&self) -> &BTreeMap<Segment<8>, FlexIdRankList> {
        &self.f
    }

    pub fn x(&self) -> &BTreeMap<Segment<8>, FlexIdRankList> {
        &self.x
    }

    pub fn y(&self) -> &BTreeMap<Segment<8>, FlexIdRankList> {
        &self.y
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FlexIdRank, &(FlexId, T))> {
        self.main.iter()
    }
}

impl<T> Default for VBitCore<T> {
    fn default() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
        }
    }
}
