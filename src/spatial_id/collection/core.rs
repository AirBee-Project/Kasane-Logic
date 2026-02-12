use roaring::RoaringTreemap;
use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry},
    hash::Hash,
};

use crate::{
    FlexId, FlexIdRank, Segment,
    spatial_id::collection::{RECYCLE_RANK_MAX, scanner::Scanner},
};

/// 空間IDの物理的な格納とインデックス管理を行う中核構造体
/// T: IDに関連付けられる追加メタデータ（TableならValueRank、Setなら()）
#[derive(Clone, Debug)]
pub struct SpatialCore<T> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,

    main: BTreeMap<FlexIdRank, (FlexId, T)>,

    next_rank: u64,
    recycle_rank: Vec<u64>,
}

impl<T> Default for SpatialCore<T> {
    fn default() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
            recycle_rank: Vec::new(),
        }
    }
}

impl<T> Scanner for SpatialCore<T> {
    fn f(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.f
    }

    fn x(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.x
    }

    fn y(&self) -> &BTreeMap<Segment, RoaringTreemap> {
        &self.y
    }
}

impl<T> SpatialCore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.main.len()
    }

    pub fn is_empty(&self) -> bool {
        self.main.is_empty()
    }

    pub fn get_entry(&self, rank: &FlexIdRank) -> Option<&(FlexId, T)> {
        self.main.get(rank)
    }

    pub fn get_flex_id(&self, rank: &FlexIdRank) -> Option<&FlexId> {
        self.main.get(rank).map(|(f, _)| f)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FlexIdRank, &(FlexId, T))> {
        self.main.iter()
    }

    /// ランクを発行し、インデックスに登録する
    /// メタデータ `meta` を一緒に保存する
    pub fn insert_entry(&mut self, flex_id: FlexId, meta: T) -> FlexIdRank {
        let rank = self.fetch_rank();

        Self::dimension_insert(&mut self.f, flex_id.as_f().clone(), rank);
        Self::dimension_insert(&mut self.x, flex_id.as_x().clone(), rank);
        Self::dimension_insert(&mut self.y, flex_id.as_y().clone(), rank);

        self.main.insert(rank, (flex_id, meta));

        rank
    }

    /// ランクを指定してインデックスから削除する
    /// 削除された (FlexId, T) を返す
    pub fn remove_entry(&mut self, rank: FlexIdRank) -> Option<(FlexId, T)> {
        if let Some((flex_id, meta)) = self.main.remove(&rank) {
            Self::dimension_remove(&mut self.f, flex_id.as_f(), rank);
            Self::dimension_remove(&mut self.x, flex_id.as_x(), rank);
            Self::dimension_remove(&mut self.y, flex_id.as_y(), rank);
            self.return_rank(rank);
            Some((flex_id, meta))
        } else {
            None
        }
    }

    fn fetch_rank(&mut self) -> FlexIdRank {
        match self.recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.next_rank;
                self.next_rank += 1;
                result
            }
        }
    }

    fn return_rank(&mut self, rank: u64) {
        if self.recycle_rank.len() < RECYCLE_RANK_MAX {
            self.recycle_rank.push(rank);
        }
    }

    fn dimension_insert(
        btree: &mut BTreeMap<Segment, RoaringTreemap>,
        segment: Segment,
        rank: FlexIdRank,
    ) {
        match btree.entry(segment) {
            Entry::Vacant(v) => {
                let mut set = RoaringTreemap::new();
                set.insert(rank);
                v.insert(set);
            }
            Entry::Occupied(mut o) => {
                o.get_mut().insert(rank);
            }
        }
    }

    fn dimension_remove(
        btree: &mut BTreeMap<Segment, RoaringTreemap>,
        segment: &Segment,
        rank: FlexIdRank,
    ) {
        if let Some(entry) = btree.get_mut(segment) {
            entry.remove(rank);
            if entry.is_empty() {
                btree.remove(segment);
            }
        }
    }
}
