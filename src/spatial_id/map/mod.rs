use roaring::RoaringTreemap;
use std::collections::BTreeMap;

use crate::spatial_id::{
    SpatialIdEncode,
    encode::{self, EncodeId},
    range::RangeId,
    segment::encode::EncodeSegment,
};
use std::ops::Bound::Excluded;

type Rank = u64;

#[derive(PartialEq, Clone, Debug)]
pub struct SpatialIdSet {
    f: BTreeMap<EncodeSegment, RoaringTreemap>,
    x: BTreeMap<EncodeSegment, RoaringTreemap>,
    y: BTreeMap<EncodeSegment, RoaringTreemap>,
    main: BTreeMap<Rank, EncodeId>,
    next_rank: Rank,
}

impl SpatialIdSet {
    ///新しく[SpatialIdSet]を作成する。
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: BTreeMap::new(),
            next_rank: 0,
        }
    }

    ///[SpatialIdSet]に含まれる[RangeId]を取り出す。
    pub fn iter(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.decode())
    }

    ///[SpatialIdSet]に含まれる[EncodeId]を取り出す。
    fn iter_encode(&self) -> impl Iterator<Item = EncodeId> + '_ {
        self.main.iter().map(|(_, encode_id)| encode_id.clone())
    }

    ///[SpatialIdSet]に入っている[EncodeId]の個数を返す。
    pub fn size(&self) -> usize {
        self.main.len()
    }

    ///[SpatialIdSet]の中にある`target`と関連ある[EncodeId]のRankを返す。
    fn related(&self, target: &EncodeId) -> RoaringTreemap {
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
        let f_related = related_segments(&self.f, target.as_f());
        let x_related = related_segments(&self.x, target.as_x());
        let y_related = related_segments(&self.y, target.as_y());

        let result_bitmap = f_related & x_related & y_related;

        result_bitmap
    }

    pub unsafe fn uncheck_insert<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            let rank = self.next_rank;
            self.next_rank += 1;

            self.f
                .entry(encode_id.as_f().clone())
                .or_default()
                .insert(rank);
            self.x
                .entry(encode_id.as_x().clone())
                .or_default()
                .insert(rank);
            self.y
                .entry(encode_id.as_y().clone())
                .or_default()
                .insert(rank);
            self.main.insert(rank, encode_id.clone());
        }
    }

    ///[SpatialIdSet]に[SpatialId]を挿入する。
    pub fn insert<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            //重なりがある部分を自身から削除する
            let mut base = SpatialIdSet::new();
            unsafe { base.uncheck_insert(&encode_id) };
            for related_rank in self.related(&encode_id) {
                let need_remove = self.main.get(&related_rank).unwrap();
                base.remove(need_remove);
            }

            //結合しながら挿入
            for need_insert in base.iter_encode() {
                self.join_insert(&need_insert);
            }
        }
    }

    ///対象のID領域を削除する。
    pub fn remove<T: SpatialIdEncode>(&mut self, target: &T) {
        for encode_id in target.encode() {
            for related_rank in self.related(&encode_id) {
                let base = self.main.remove(&related_rank).unwrap();
                self.f.remove(base.as_f());
                self.x.remove(base.as_x());
                self.y.remove(base.as_y());
                let diff = base.difference(&encode_id);
                for need_insert in diff {
                    self.join_insert(&need_insert)
                }
            }
        }
    }

    pub fn difference(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result;
        if self.size() < other.size() {
            result = other.clone();
            for encode in self.iter_encode() {
                result.remove(&encode);
            }
        } else {
            result = self.clone();
            for encode in other.iter_encode() {
                result.remove(&encode);
            }
        };
        result
    }

    pub fn get<T: SpatialIdEncode>(&mut self, target: &T) -> EncodeId {
        todo!()
    }

    pub fn intersection(&self, other: &SpatialIdSet) -> SpatialIdSet {
        todo!()
    }

    pub fn union(&self, other: &SpatialIdSet) -> SpatialIdSet {
        let mut result;
        if self.size() < other.size() {
            result = other.clone();
            for encode in self.iter_encode() {
                result.insert(&encode);
            }
        } else {
            result = self.clone();
            for encode in other.iter_encode() {
                result.insert(&encode);
            }
        };
        result
    }

    /// IDを追加し、可能な場合は結合を行う
    fn join_insert(&mut self, target: &EncodeId) {
        //Fの兄弟候補
        let f_sibling = EncodeId::new(
            target.as_f().sibling(),
            target.as_x().clone(),
            target.as_y().clone(),
        );

        //Fで結合
        match self.find_encode(&f_sibling) {
            Some(rank) => {
                self.remove_rank(rank);
                self.join_insert(&EncodeId::new(
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
        match self.find_encode(&x_sibling) {
            Some(rank) => {
                self.remove_rank(rank);
                self.join_insert(&EncodeId::new(
                    target.as_f().clone(),
                    target.as_x().parent().unwrap(),
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
        match self.find_encode(&y_sibling) {
            Some(rank) => {
                self.remove_rank(rank);
                self.join_insert(&EncodeId::new(
                    target.as_f().clone(),
                    target.as_x().clone(),
                    target.as_y().parent().unwrap(),
                ));
                return;
            }
            None => {}
        }

        unsafe { self.uncheck_insert(target) };
    }

    ///指定されたEncodeIdと完全に一致するEncodeIdのRankを返す
    fn find_encode(&self, target: &EncodeId) -> Option<Rank> {
        let f_hits = self.f.get(target.as_f())?;
        let x_hits = self.x.get(target.as_x())?;
        let y_hits = self.y.get(target.as_y())?;
        let result = f_hits & x_hits & y_hits;
        result.iter().next()
    }

    /// 指定されたRankを持つIDを全てのインデックスから完全に削除する
    fn remove_rank(&mut self, rank: Rank) {
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
