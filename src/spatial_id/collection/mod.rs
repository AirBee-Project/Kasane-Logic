use roaring::RoaringTreemap;

use crate::{
    BTreeMapTrait,
    spatial_id::{flex_id::FlexId, segment::Segment},
};
use std::ops::Bound::{Excluded, Included};

pub mod set;
pub mod table;

pub type FlexIdRank = u64;
pub type ValueRank = u64;

///Rankの
const MAX_RECYCLE_CAPACITY: usize = 1024;

#[derive(Debug, Default)]
pub struct Plan {
    add: Vec<FlexId>,
    remove: Vec<FlexIdRank>,
}

pub trait Collection {
    type Main: BTreeMapTrait<FlexIdRank, FlexId>;
    type Dimension: BTreeMapTrait<Segment, RoaringTreemap>;
    fn main(&self) -> &Self::Main;
    fn main_mut(&mut self) -> &mut Self::Main;

    fn f(&self) -> &Self::Dimension;
    fn f_mut(&mut self) -> &mut Self::Dimension;
    fn x(&self) -> &Self::Dimension;
    fn x_mut(&mut self) -> &mut Self::Dimension;
    fn y(&self) -> &Self::Dimension;
    fn y_mut(&mut self) -> &mut Self::Dimension;

    fn fetch_rank(&mut self) -> u64;
    fn return_rank(&mut self, rank: u64);

    /// FlexIdRankに割り当てられていたFlexIdを削除し、そのFlexIdを返す
    fn remove_flex_id(&mut self, rank: FlexIdRank) -> Option<FlexId> {
        let flex_id = self.main_mut().remove(&rank)?;
        let remove_from_dim = |map: &mut Self::Dimension, seg: &Segment| {
            if let Some(bitmap) = map.get_mut(seg) {
                bitmap.remove(rank);
                if bitmap.is_empty() {
                    map.remove(seg);
                }
            }
        };
        remove_from_dim(self.f_mut(), flex_id.as_f());
        remove_from_dim(self.x_mut(), flex_id.as_x());
        remove_from_dim(self.y_mut(), flex_id.as_y());
        self.return_rank(rank);
        Some(flex_id)
    }

    ///FlexIdを挿入し、割り当てられたFlexIdRankを返す
    fn insert_flex_id(&mut self, target: &FlexId) -> FlexIdRank {
        let rank = self.fetch_rank();
        let update_dim = |map: &mut Self::Dimension, seg: Segment| {
            map.get_or_insert_with(seg, RoaringTreemap::new)
                .insert(rank);
        };
        update_dim(self.f_mut(), target.as_f().clone());
        update_dim(self.x_mut(), target.as_x().clone());
        update_dim(self.y_mut(), target.as_y().clone());
        self.main_mut().insert(rank, target.clone());
        rank
    }

    ///あるFlexIdRankの実体のFlexIdを参照する
    /// 見つからない場合はNoneを返す
    fn get_flex_id(&self, flex_id_rank: FlexIdRank) -> Option<&FlexId> {
        self.main().get(&flex_id_rank)
    }

    ///あるFlexIdとf方向で兄弟なFlexIdのRankを取得する
    fn get_f_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(&target.as_f().sibling())?;
        let x_ranks = self.x().get(target.as_x())?;
        let y_ranks = self.y().get(target.as_y())?;
        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    ///あるFlexIdとx方向で兄弟なFlexIdのRankを取得する
    fn get_x_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(&target.as_f())?;
        let x_ranks = self.x().get(&target.as_x().sibling())?;
        let y_ranks = self.y().get(target.as_y())?;
        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    ///あるFlexIdとy方向で兄弟なFlexIdのRankを取得する
    fn get_y_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(&target.as_f())?;
        let x_ranks = self.x().get(target.as_x())?;
        let y_ranks = self.y().get(&target.as_y().sibling())?;
        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    ///FlexIdの全ての参照を返す
    fn flex_ids(&self) -> impl Iterator<Item = &FlexId> {
        self.main().iter().map(|f| f.1)
    }

    ///FlexIdの可変参照を返す
    fn flex_ids_mut(&mut self) -> impl Iterator<Item = &mut FlexId> {
        self.main_mut().iter_mut().map(|f| f.1)
    }

    ///あるFlexIdと関連のあるFlexIdRankを全て返す
    fn related(&self, target: &FlexId) -> RoaringTreemap {
        let get_related_segment = |map: &Self::Dimension, seg: &Segment| -> RoaringTreemap {
            let mut bitmap = RoaringTreemap::new();
            let mut current = seg.parent();
            while let Some(parent) = current {
                if let Some(ranks) = map.get(&parent) {
                    bitmap |= ranks;
                }
                current = parent.parent();
            }
            let end = seg.descendant_range_end();
            for (_, ranks) in map.range((Included(seg), Excluded(&end))) {
                bitmap |= ranks;
            }

            bitmap
        };
        let f_related = get_related_segment(self.f(), target.as_f());
        let x_related = get_related_segment(self.x(), target.as_x());
        let y_related = get_related_segment(self.y(), target.as_y());

        let intersection = f_related & x_related & y_related;
        intersection.into_iter().collect()
    }
}
