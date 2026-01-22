use roaring::RoaringTreemap;

use crate::spatial_id::{flex_id::FlexId, segment::Segment};
use crate::storage::{Batch, KeyValueStore, OrderedKeyValueStore};

pub mod set;
pub mod table;

pub(crate) type FlexIdRank = u64;
pub(crate) type ValueRank = u64;

/// Rankのごみ箱のキャパシティー
const MAX_RECYCLE_CAPACITY: usize = 1024;

pub trait Collection {
    type Main: KeyValueStore<FlexIdRank, FlexId>;
    type Dimension: OrderedKeyValueStore<Segment, RoaringTreemap>;

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
        let flex_id = self.main().get(&rank)?;

        let mut main_batch = Batch::new();
        main_batch.delete(rank);

        let mut update_dim = |store: &mut Self::Dimension, seg: &Segment| {
            let mut batch = Batch::new();
            // 現在の状態を取得
            if let Some(mut bitmap) = store.get(seg) {
                // 変更を適用
                let changed = bitmap.remove(rank);

                // Write: 変更があればバッチに追加
                if bitmap.is_empty() {
                    batch.delete(seg.clone());
                } else if changed {
                    batch.put(seg.clone(), bitmap);
                }
            }
            store.apply_batch(batch);
        };

        update_dim(self.f_mut(), flex_id.as_f());
        update_dim(self.x_mut(), flex_id.as_x());
        update_dim(self.y_mut(), flex_id.as_y());

        // 3. Apply: Mainへの変更を適用
        self.main_mut().apply_batch(main_batch);

        self.return_rank(rank);
        Some(flex_id)
    }

    /// FlexIdを挿入し、割り当てられたFlexIdRankを返す
    fn insert_flex_id(&mut self, target: &FlexId) -> FlexIdRank {
        let rank = self.fetch_rank();

        let update_dim = |store: &mut Self::Dimension, seg: Segment| {
            let mut batch = Batch::new();
            let mut bitmap = store.get(&seg).unwrap_or_else(RoaringTreemap::new);
            bitmap.insert(rank);
            batch.put(seg, bitmap);
            store.apply_batch(batch);
        };

        update_dim(self.f_mut(), target.as_f().clone());
        update_dim(self.x_mut(), target.as_x().clone());
        update_dim(self.y_mut(), target.as_y().clone());

        let mut main_batch = Batch::new();
        main_batch.put(rank, target.clone());
        self.main_mut().apply_batch(main_batch);

        rank
    }

    /// あるFlexIdRankの実体のFlexIdを参照する
    fn get_flex_id(&self, flex_id_rank: FlexIdRank) -> Option<FlexId> {
        self.main().get(&flex_id_rank)
    }

    /// あるFlexIdとf方向で兄弟なFlexIdのRankを取得する
    fn get_f_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        // get() が Option<V> を返すので ? でチェーンできる
        let f_ranks = self.f().get(&target.as_f().sibling())?;
        let x_ranks = self.x().get(target.as_x())?;
        let y_ranks = self.y().get(target.as_y())?;

        // RoaringTreemap同士の積集合
        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    /// あるFlexIdとx方向で兄弟なFlexIdのRankを取得する
    fn get_x_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(target.as_f())?;
        let x_ranks = self.x().get(&target.as_x().sibling())?;
        let y_ranks = self.y().get(target.as_y())?;

        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    /// あるFlexIdとy方向で兄弟なFlexIdのRankを取得する
    fn get_y_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(target.as_f())?;
        let x_ranks = self.x().get(target.as_x())?;
        let y_ranks = self.y().get(&target.as_y().sibling())?;

        let intersection = f_ranks & x_ranks & y_ranks;
        intersection.iter().next()
    }

    fn flex_ids(&self) -> Box<dyn Iterator<Item = FlexId> + '_> {
        Box::new(self.main().iter().map(|(_, v)| v))
    }

    /// あるFlexIdと関連のあるFlexIdRankを全て返す
    fn related(&self, target: &FlexId) -> RoaringTreemap {
        let get_related_segment = |store: &Self::Dimension, seg: &Segment| -> RoaringTreemap {
            let mut bitmap = RoaringTreemap::new();
            let mut current = seg.parent();
            while let Some(parent) = current {
                if let Some(ranks) = store.get(&parent) {
                    bitmap |= ranks;
                }
                current = parent.parent();
            }
            let end = seg.descendant_range_end();
            let iter: Box<dyn Iterator<Item = (Segment, RoaringTreemap)>> = match end {
                Some(end_segment) => {
                    if seg <= &end_segment {
                        store.scan(seg..=&end_segment)
                    } else {
                        println!("{}", seg);
                        println!("{}", end_segment);
                        store.scan(seg..)
                    }
                }
                None => store.scan(seg..),
            };

            for (_, ranks) in iter {
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
