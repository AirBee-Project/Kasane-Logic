use roaring::RoaringTreemap;

use crate::SetOnMemory;
use crate::spatial_id::{flex_id::FlexId, segment::Segment};
use crate::storage::{Batch, KeyValueStore, OrderedKeyValueStore};

pub mod set;
pub mod table;

pub type FlexIdRank = u64;
pub type ValueRank = u64;

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

    fn fetch_flex_rank(&mut self) -> u64;
    fn return_flex_rank(&mut self, rank: u64);

    ///ストレージ間でデータを移動するときに次に割り当てるべきRankを引き継ぐ用
    fn move_flex_rank(&self) -> u64;

    ///ストレージ間でデータを移動するときにゴミのRankを引き継ぐ用
    fn move_flex_rank_free_list(&self) -> Vec<u64>;

    /// FlexIdRankに割り当てられていたFlexIdを削除し、そのFlexIdを返す
    async fn remove_flex_id(&mut self, rank: FlexIdRank) -> Option<FlexId> {
        let flex_id = self.main().get(&rank).await?.clone();

        let mut main_batch = Batch::new();
        main_batch.delete(rank);

        // Update F dimension
        {
            let seg = flex_id.as_f();
            let mut batch = Batch::new();
            if let Some(bitmap) = self.f().get(seg).await {
                let mut bitmap_owned = bitmap.clone();
                let changed = bitmap_owned.remove(rank);
                if bitmap_owned.is_empty() {
                    batch.delete(seg.clone());
                } else if changed {
                    batch.put(seg.clone(), bitmap_owned);
                }
            }
            self.f_mut().apply_batch(batch).await;
        }

        // Update X dimension
        {
            let seg = flex_id.as_x();
            let mut batch = Batch::new();
            if let Some(bitmap) = self.x().get(seg).await {
                let mut bitmap_owned = bitmap.clone();
                let changed = bitmap_owned.remove(rank);
                if bitmap_owned.is_empty() {
                    batch.delete(seg.clone());
                } else if changed {
                    batch.put(seg.clone(), bitmap_owned);
                }
            }
            self.x_mut().apply_batch(batch).await;
        }

        // Update Y dimension
        {
            let seg = flex_id.as_y();
            let mut batch = Batch::new();
            if let Some(bitmap) = self.y().get(seg).await {
                let mut bitmap_owned = bitmap.clone();
                let changed = bitmap_owned.remove(rank);
                if bitmap_owned.is_empty() {
                    batch.delete(seg.clone());
                } else if changed {
                    batch.put(seg.clone(), bitmap_owned);
                }
            }
            self.y_mut().apply_batch(batch).await;
        }

        self.main_mut().apply_batch(main_batch).await;

        self.return_flex_rank(rank);
        Some(flex_id)
    }

    /// ターゲットとなるFlexIdと空間的に重複する既存のIDを検出し、削除する。
    /// 戻り値として、「削除されたIDのRank」と「そのIDから生成された破片(Fragments)」のリストを返す。
    async fn resolve_collisions(&mut self, target: &FlexId) -> Vec<(FlexIdRank, Vec<FlexId>)> {
        let mut collisions = Vec::new();
        let related_ranks: Vec<FlexIdRank> = self.related(target).await.into_iter().collect();

        for rank in related_ranks {
            if let Some(existing_id) = self.get_flex_id(rank).await {
                if target.intersection(&existing_id).is_some() {
                    let existing_backup = existing_id.clone();
                    // 既存のIDを削除
                    self.remove_flex_id(rank).await;
                    // 削られた断片を計算
                    let fragments = existing_backup.difference(target);

                    collisions.push((rank, fragments));
                }
            }
        }
        collisions
    }
    /// FlexIdを挿入し、割り当てられたFlexIdRankを返す
    async fn insert_flex_id(&mut self, target: &FlexId) -> FlexIdRank {
        let rank = self.fetch_flex_rank();

        // Update F dimension
        {
            let seg = target.as_f().clone();
            let mut batch = Batch::new();
            let bitmap_owned = {
                let bitmap = self.f().get(&seg).await;
                bitmap.as_ref().map(|b| (**b).clone()).unwrap_or_else(RoaringTreemap::new)
            };
            let mut bitmap_owned = bitmap_owned;
            bitmap_owned.insert(rank);
            batch.put(seg, bitmap_owned);
            self.f_mut().apply_batch(batch).await;
        }

        // Update X dimension
        {
            let seg = target.as_x().clone();
            let mut batch = Batch::new();
            let bitmap_owned = {
                let bitmap = self.x().get(&seg).await;
                bitmap.as_ref().map(|b| (**b).clone()).unwrap_or_else(RoaringTreemap::new)
            };
            let mut bitmap_owned = bitmap_owned;
            bitmap_owned.insert(rank);
            batch.put(seg, bitmap_owned);
            self.x_mut().apply_batch(batch).await;
        }

        // Update Y dimension
        {
            let seg = target.as_y().clone();
            let mut batch = Batch::new();
            let bitmap_owned = {
                let bitmap = self.y().get(&seg).await;
                bitmap.as_ref().map(|b| (**b).clone()).unwrap_or_else(RoaringTreemap::new)
            };
            let mut bitmap_owned = bitmap_owned;
            bitmap_owned.insert(rank);
            batch.put(seg, bitmap_owned);
            self.y_mut().apply_batch(batch).await;
        }

        let mut main_batch = Batch::new();
        main_batch.put(rank, target.clone());
        self.main_mut().apply_batch(main_batch).await;

        rank
    }

    /// あるFlexIdRankの実体のFlexIdを参照する
    async fn get_flex_id(&self, flex_id_rank: FlexIdRank) -> Option<FlexId> {
        self.main().get(&flex_id_rank).await.map(|f| f.clone())
    }

    /// あるFlexIdとf方向で兄弟なFlexIdのRankを取得する
    async fn get_f_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(&target.as_f().sibling()).await?;
        let x_ranks = self.x().get(target.as_x()).await?;
        let y_ranks = self.y().get(target.as_y()).await?;
        let intersection = &*f_ranks & &*x_ranks & &*y_ranks;
        intersection.iter().next()
    }

    /// あるFlexIdとx方向で兄弟なFlexIdのRankを取得する
    async fn get_x_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(target.as_f()).await?;
        let x_ranks = self.x().get(&target.as_x().sibling()).await?;
        let y_ranks = self.y().get(target.as_y()).await?;

        let intersection = &*f_ranks & &*x_ranks & &*y_ranks;
        intersection.iter().next()
    }

    /// あるFlexIdとy方向で兄弟なFlexIdのRankを取得する
    async fn get_y_sibling_flex_id(&self, target: &FlexId) -> Option<FlexIdRank> {
        let f_ranks = self.f().get(target.as_f()).await?;
        let x_ranks = self.x().get(target.as_x()).await?;
        let y_ranks = self.y().get(&target.as_y().sibling()).await?;

        let intersection = &*f_ranks & &*x_ranks & &*y_ranks;
        intersection.iter().next()
    }

    fn flex_ids(&self) -> Box<dyn Iterator<Item = FlexId> + '_> {
        Box::new(self.main().iter().map(|(_, v)| v))
    }

    /// あるFlexIdと関連のあるFlexIdRankを全て返す
    async fn related(&self, target: &FlexId) -> RoaringTreemap {
        let mut f_related = RoaringTreemap::new();
        let mut current = target.as_f().parent();
        while let Some(parent) = current {
            if let Some(ranks) = self.f().get(&parent).await {
                f_related |= &*ranks;
            }
            current = parent.parent();
        }
        let end = target.as_f().descendant_range_end();
        let f_iter: Box<dyn Iterator<Item = (Segment, RoaringTreemap)>> = match end {
            Some(end_segment) => {
                if target.as_f() <= &end_segment {
                    self.f().scan(target.as_f()..=&end_segment)
                } else {
                    println!("{}", target.as_f());
                    println!("{}", end_segment);
                    self.f().scan(target.as_f()..)
                }
            }
            None => self.f().scan(target.as_f()..),
        };
        for (_, ranks) in f_iter {
            f_related |= ranks;
        }

        let mut x_related = RoaringTreemap::new();
        let mut current = target.as_x().parent();
        while let Some(parent) = current {
            if let Some(ranks) = self.x().get(&parent).await {
                x_related |= &*ranks;
            }
            current = parent.parent();
        }
        let end = target.as_x().descendant_range_end();
        let x_iter: Box<dyn Iterator<Item = (Segment, RoaringTreemap)>> = match end {
            Some(end_segment) => {
                if target.as_x() <= &end_segment {
                    self.x().scan(target.as_x()..=&end_segment)
                } else {
                    println!("{}", target.as_x());
                    println!("{}", end_segment);
                    self.x().scan(target.as_x()..)
                }
            }
            None => self.x().scan(target.as_x()..),
        };
        for (_, ranks) in x_iter {
            x_related |= ranks;
        }

        let mut y_related = RoaringTreemap::new();
        let mut current = target.as_y().parent();
        while let Some(parent) = current {
            if let Some(ranks) = self.y().get(&parent).await {
                y_related |= &*ranks;
            }
            current = parent.parent();
        }
        let end = target.as_y().descendant_range_end();
        let y_iter: Box<dyn Iterator<Item = (Segment, RoaringTreemap)>> = match end {
            Some(end_segment) => {
                if target.as_y() <= &end_segment {
                    self.y().scan(target.as_y()..=&end_segment)
                } else {
                    println!("{}", target.as_y());
                    println!("{}", end_segment);
                    self.y().scan(target.as_y()..)
                }
            }
            None => self.y().scan(target.as_y()..),
        };
        for (_, ranks) in y_iter {
            y_related |= ranks;
        }

        let intersection = f_related & x_related & y_related;
        intersection.into_iter().collect()
    }

    fn to_set(&self) -> SetOnMemory {
        let mut set = SetOnMemory::default();
        for flex_id in self.flex_ids() {
            set.insert(&flex_id);
        }
        set
    }
}
