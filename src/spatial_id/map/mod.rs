use roaring::RoaringTreemap;
use std::collections::BTreeMap;

use crate::spatial_id::{SpatialId, encode::EncodeId, segment::encode::EncodeSegment};
use std::ops::Bound::Excluded;

type Rank = u64;

pub struct SpatialIdMap {
    f: BTreeMap<EncodeSegment, RoaringTreemap>,
    x: BTreeMap<EncodeSegment, RoaringTreemap>,
    y: BTreeMap<EncodeSegment, RoaringTreemap>,
    main: BTreeMap<Rank, EncodeId>,
    next_rank: Rank,
}

impl SpatialIdMap {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
        }
    }

    pub fn insert<T: SpatialId>(spatial_id: T) {}

    pub fn find_related(&self, encode_id: EncodeId) -> RoaringTreemap {
        let get_related_ranks = |map: &BTreeMap<EncodeSegment, RoaringTreemap>,
                                 target: &EncodeSegment|
         -> RoaringTreemap {
            let mut related_bitmap = RoaringTreemap::new();
            let mut current = Some(target.clone());
            while let Some(seg) = current {
                if let Some(ranks) = map.get(&seg) {
                    // 和集合 (OR) をとる
                    related_bitmap |= ranks;
                }
                current = seg.parent();
            }
            let range_end = target.descendant_range_end();
            for (_, ranks) in map.range((Excluded(target), Excluded(&range_end))) {
                // 和集合 (OR) をとる
                related_bitmap |= ranks;
            }

            related_bitmap
        };

        let f_related = get_related_ranks(&self.f, encode_id.as_f());
        let x_related = get_related_ranks(&self.x, encode_id.as_x());
        let y_related = get_related_ranks(&self.y, encode_id.as_y());
        let result_bitmap = f_related & x_related & y_related;

        result_bitmap
    }

    /// IDを追加し、可能な場合は結合を行う
    pub fn add(&mut self, mut target: EncodeId) {
        //Fの兄弟候補
        let f_sibling = EncodeId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );

        match self.find(&f_sibling) {
            Some(_) => {}
            None => {}
        }

        todo!()
    }

    ///指定されたEncodeIdと完全に一致するEncodeIdのRankを返す
    fn find(&self, target: &EncodeId) -> Option<Rank> {
        let f_hits = self.f.get(target.as_f())?;
        let x_hits = self.x.get(target.as_x())?;
        let y_hits = self.y.get(target.as_y())?;
        let result = f_hits & x_hits & y_hits;
        result.iter().next()
    }

    /// 指定されたRankを持つIDを全てのインデックスから完全に削除する
    pub fn delete(&mut self, rank: Rank) {
        let encode_id = match self.main.remove(&rank) {
            Some(v) => v,
            None => return,
        };

        let remove_from_dim = |map: &mut BTreeMap<EncodeSegment, RoaringTreemap>,
                               segment: EncodeSegment| {
            if let std::collections::btree_map::Entry::Occupied(mut entry) = map.entry(segment) {
                let bitmap = entry.get_mut();
                bitmap.remove(rank);
                if bitmap.is_empty() {
                    entry.remove_entry();
                }
            }
        };

        remove_from_dim(&mut self.f, encode_id.as_f().clone());
        remove_from_dim(&mut self.x, encode_id.as_x().clone());
        remove_from_dim(&mut self.y, encode_id.as_y().clone());
    }
}
