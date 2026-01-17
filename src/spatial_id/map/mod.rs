use roaring::RoaringTreemap;
use std::collections::BTreeMap;

use crate::spatial_id::{
    SpatialId, encode::EncodeId, range::RangeId, segment::encode::EncodeSegment,
};
use std::ops::Bound::Excluded;

type Rank = u64;

#[derive(PartialEq, Clone)]
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

    //デコードする関数
    pub fn iter(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.decode())
    }

    pub(crate) fn iter_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.clone())
    }

    ///入っているEncodeIdの個数
    pub fn size(&self) -> usize {
        self.main.len()
    }

    pub fn insert<T: SpatialId>(&mut self, spatial_id: T) {
        for encode in spatial_id.encode() {
            self.insert_encode(encode);
        }
    }

    fn insert_encode(&mut self, encode_id: EncodeId) {}

    pub fn intersection(&self, other: &SpatialIdMap) -> SpatialIdMap {
        todo!()
    }

    pub fn union(&self, other: &SpatialIdMap) -> SpatialIdMap {
        let mut result;
        if self.size() < other.size() {
            result = other.clone();
            for encode in self.iter_encode() {
                result.insert_encode(encode);
            }
        } else {
            result = self.clone();
            for encode in other.iter_encode() {
                result.insert_encode(encode);
            }
        };
        result
    }

    pub fn difference(&self, other: &SpatialIdMap) -> SpatialIdMap {
        todo!()
    }

    ///関連ある[EncodeId]のRankのリストを返す
    pub fn find_related(&self, encode_id: EncodeId) -> RoaringTreemap {
        let related_segments = |map: &BTreeMap<EncodeSegment, RoaringTreemap>,
                                target: &EncodeSegment|
         -> RoaringTreemap {
            let mut related_bitmap = RoaringTreemap::new();
            let mut current = Some(target.clone());
            while let Some(seg) = current {
                if let Some(ranks) = map.get(&seg) {
                    related_bitmap |= ranks;
                }
                current = seg.parent();
            }
            let range_end = target.descendant_range_end();
            for (_, ranks) in map.range((Excluded(target), Excluded(&range_end))) {
                related_bitmap |= ranks;
            }
            related_bitmap
        };
        let f_related = related_segments(&self.f, encode_id.as_f());
        let x_related = related_segments(&self.x, encode_id.as_x());
        let y_related = related_segments(&self.y, encode_id.as_y());
        let result_bitmap = f_related & x_related & y_related;
        result_bitmap
    }

    /// IDを追加し、可能な場合は結合を行う
    pub fn add(&mut self, target: EncodeId) {
        //Fの兄弟候補
        let f_sibling = EncodeId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );

        //Fで結合
        match self.find(&f_sibling) {
            Some(rank) => {
                self.delete(rank);
                self.add(EncodeId::new(
                    target.as_f().parent().unwrap(),
                    target.as_x().clone(),
                    target.as_y().clone(),
                ));
                return;
            }
            None => {}
        }

        //Xの兄弟候補
        let x_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().sibling(),
            target.as_y().clone(),
        );

        //Xで結合
        match self.find(&x_sibling) {
            Some(rank) => {
                self.delete(rank);
                self.add(EncodeId::new(
                    target.as_f().clone(),
                    target.as_x().sibling(),
                    target.as_y().clone(),
                ));
                return;
            }
            None => {}
        }

        //Yの兄弟候補
        let y_sibling = EncodeId::new(
            target.as_f().clone(),
            target.as_x().clone(),
            target.as_y().sibling(),
        );

        //Yで結合
        match self.find(&y_sibling) {
            Some(rank) => {
                self.delete(rank);
                self.add(EncodeId::new(
                    target.as_f().clone(),
                    target.as_x().clone(),
                    target.as_y().sibling(),
                ));
                return;
            }
            None => {}
        }

        let rank = self.next_rank;
        self.next_rank += 1;

        self.f
            .entry(target.as_f().clone())
            .or_default()
            .insert(rank);
        self.x
            .entry(target.as_x().clone())
            .or_default()
            .insert(rank);
        self.y
            .entry(target.as_y().clone())
            .or_default()
            .insert(rank);

        self.main.insert(rank, target);
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
    fn delete(&mut self, rank: Rank) {
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
