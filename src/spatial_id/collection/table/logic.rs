use std::fmt::{self, Display};

use crate::{
    KeyValueStore, RangeId, SetOnMemory, SingleId,
    spatial_id::{
        ToFlexId,
        collection::{
            Collection, FlexIdRank, ValueRank,
            table::{TableStorage, memory::TableOnMemory},
        },
        flex_id::FlexId,
    },
    storage::Batch,
};
use std::fmt::Debug;

#[derive(Default)]
pub struct TableLogic<S: TableStorage + Collection>(pub S);

impl<S> TableLogic<S>
where
    S: TableStorage + Collection,
{
    ///TableStorageが実装された型を開いて、操作可能な状態にする
    pub fn open(table_storage: S) -> Self {
        Self(table_storage)
    }

    ///TableStorageが実装された型を読み込んで、コピーのTableOnMemoryを作成する
    ///大量のReadが発生する可能性があるため、注意して使用せよ
    pub fn load(table_storage: &S) -> TableOnMemory<S::Value> {
        todo!()
    }

    ///SetStorageが実装された型を外に出す
    pub fn close(self) -> S {
        self.0
    }

    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn range_ids_values(&self) -> impl Iterator<Item = (RangeId, S::Value)> + '_ {
        self.flex_ids_values()
            .map(|(flex_id, value)| (flex_id.range_id(), value))
    }

    pub fn single_ids_values(&self) -> impl Iterator<Item = (SingleId, S::Value)> + '_ {
        self.range_ids_values().flat_map(|(range_id, value)| {
            let singles: Vec<SingleId> = range_id.single_ids().collect();
            singles
                .into_iter()
                .map(move |single_id| (single_id, value.clone()))
        })
    }

    pub(crate) fn flex_ids_values(&self) -> Box<dyn Iterator<Item = (FlexId, S::Value)> + '_> {
        // Collect forward and dictionary maps into Vecs to avoid async in iterator
        let forward_map: std::collections::HashMap<FlexIdRank, ValueRank> = self.0.forward().iter().collect();
        let dict_map: std::collections::HashMap<ValueRank, S::Value> = self.0.dictionary().iter().collect();
        
        Box::new(self.0.main().iter().filter_map(move |(rank, flex_id)| {
            let val_rank = forward_map.get(&rank)?;
            let value = dict_map.get(val_rank)?;
            Some((flex_id.clone(), value.clone()))
        }))
    }

    ///内部にあるIDを全てRangeIdとして返す
    pub fn range_ids(&self) -> impl Iterator<Item = RangeId> + '_ {
        self.0.flex_ids().map(|flex_id| flex_id.range_id())
    }

    pub fn single_ids(&self) -> impl Iterator<Item = SingleId> + '_ {
        self.0
            .flex_ids()
            .flat_map(|flex_id| flex_id.range_id().single_ids().collect::<Vec<_>>())
    }

    pub(crate) fn flex_ids(&self) -> Box<dyn Iterator<Item = FlexId> + '_> {
        self.0.flex_ids()
    }

    ///値が
    pub async fn insert<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        for new_id in target.flex_ids() {
            let collisions = self.0.resolve_collisions(&new_id).await;

            for (removed_rank, fragments) in collisions {
                let old_value_opt = if let Some(val_rank) = self.0.forward().get(&removed_rank).await {
                    self.0.dictionary().get(&val_rank).await.map(|v| v.clone())
                } else {
                    None
                };

                let mut batch = Batch::new();
                batch.delete(removed_rank);
                self.0.forward_mut().apply_batch(batch).await;

                if let Some(old_value) = old_value_opt {
                    for frag in fragments {
                        unsafe { self.join_insert_unchecked(&frag, &old_value).await };
                    }
                }
            }

            unsafe { self.join_insert_unchecked(&new_id, value).await };
        }
    }

    pub async fn get<I: ToFlexId>(&mut self, target: &I) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id).await {
                let related_id = self.0.get_flex_id(related_rank).await.unwrap();
                let value_rank = self.0.forward().get(&related_rank).await.unwrap();
                let value = self.0.dictionary().get(&value_rank).await.unwrap();
                result.insert(&flex_id.intersection(&related_id).unwrap(), &value.clone()).await;
            }
        }
        result
    }

    pub async fn remove<I: ToFlexId>(&mut self, target: &I) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id).await {
                let related_id = self.0.get_flex_id(related_rank).await.unwrap();
                let value_rank = self.0.forward().get(&related_rank).await.unwrap();
                let value = self.0.dictionary().get(&value_rank).await.unwrap();
                for removed_flex_id in flex_id.difference(&related_id) {
                    result.insert(&removed_flex_id, &value.clone()).await;
                }
            }
        }
        result
    }

    pub async fn get_by_value(&self, value: &S::Value) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();
        if let Some(target_val_rank) = self.0.reverse().get(value).await {
            let target_val_rank = target_val_rank.clone();
            for (flex_id_rank, val_rank) in self.0.forward().iter() {
                if val_rank == target_val_rank {
                    if let Some(flex_id) = self.0.get_flex_id(flex_id_rank).await {
                        unsafe { result.insert_unchecked(&flex_id, value).await };
                    }
                }
            }
        }
        result
    }

    pub async fn remove_by_value(&mut self, value: &S::Value) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();

        let target_val_rank = if let Some(rank) = self.0.reverse().get(value).await {
            rank.clone()
        } else {
            return result;
        };

        let mut remove_targets = Vec::new();

        for (flex_id_rank, val_rank) in self.0.forward().iter() {
            if val_rank == target_val_rank {
                remove_targets.push(flex_id_rank);
            }
        }

        for rank in remove_targets {
            if let Some(flex_id) = self.0.remove_flex_id(rank).await {
                let mut batch = Batch::new();
                batch.delete(rank);
                self.0.forward_mut().apply_batch(batch).await;
                unsafe { result.insert_unchecked(&flex_id, value).await };
            }
        }

        result
    }

    ///重複確認なく挿入を行う
    /// 結合の最適化を行わないとEqなどが正常に動作しなくなる
    /// 結合最適化を行ったものを入れないと、ロジックが壊れる
    /// もしくは、明らかに結合不能なIDなど
    pub async unsafe fn insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        let mut flex_id_ranks = Vec::new();
        for flex_id in target.flex_ids() {
            let rank = self.0.insert_flex_id(&flex_id).await;
            flex_id_ranks.push(rank);
        }
        self.0.insert_value(value, flex_id_ranks).await;
    }

    pub async unsafe fn join_insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        for flex_id in target.flex_ids() {
            // F方向の結合チェック
            if let Some(sibling_rank) = self.0.get_f_sibling_flex_id(&flex_id).await {
                // Check if values are the same
                let is_same = if let Some(val_rank) = self.0.forward().get(&sibling_rank).await {
                    if let Some(v) = self.0.dictionary().get(&val_rank).await {
                        *v == *value
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_same {
                    let sibling_id = self.0.get_flex_id(sibling_rank).await.unwrap().clone();
                    if let Some(parent) = sibling_id.f_parent() {
                        // Remove sibling
                        self.0.remove_flex_id(sibling_rank).await;
                        let mut batch = Batch::new();
                        batch.delete(sibling_rank);
                        self.0.forward_mut().apply_batch(batch).await;
                        // Recursive call
                        unsafe { Box::pin(self.join_insert_unchecked(&parent, value)).await };
                        continue;
                    }
                }
            }

            // X方向の結合チェック
            if let Some(sibling_rank) = self.0.get_x_sibling_flex_id(&flex_id).await {
                // Check if values are the same
                let is_same = if let Some(val_rank) = self.0.forward().get(&sibling_rank).await {
                    if let Some(v) = self.0.dictionary().get(&val_rank).await {
                        *v == *value
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_same {
                    let sibling_id = self.0.get_flex_id(sibling_rank).await.unwrap().clone();
                    if let Some(parent) = sibling_id.x_parent() {
                        // Remove sibling
                        self.0.remove_flex_id(sibling_rank).await;
                        let mut batch = Batch::new();
                        batch.delete(sibling_rank);
                        self.0.forward_mut().apply_batch(batch).await;
                        unsafe { Box::pin(self.join_insert_unchecked(&parent, value)).await };
                        continue;
                    }
                }
            }

            // Y方向の結合チェック
            if let Some(sibling_rank) = self.0.get_y_sibling_flex_id(&flex_id).await {
                // Check if values are the same
                let is_same = if let Some(val_rank) = self.0.forward().get(&sibling_rank).await {
                    if let Some(v) = self.0.dictionary().get(&val_rank).await {
                        *v == *value
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_same {
                    let sibling_id = self.0.get_flex_id(sibling_rank).await.unwrap().clone();
                    if let Some(parent) = sibling_id.y_parent() {
                        // Remove sibling
                        self.0.remove_flex_id(sibling_rank).await;
                        let mut batch = Batch::new();
                        batch.delete(sibling_rank);
                        self.0.forward_mut().apply_batch(batch).await;
                        unsafe { Box::pin(self.join_insert_unchecked(&parent, value)).await };
                        continue;
                    }
                }
            }

            unsafe { self.insert_unchecked(&flex_id, value).await };
        }
    }

    pub fn to_set(&self) -> SetOnMemory {
        self.0.to_set()
    }
}

impl<S> Display for TableLogic<S>
where
    S: TableStorage + Collection + Default,
    S::Value: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (range_id, value) in self.range_ids_values() {
            write!(f, "{}: {},", range_id, value)?;
        }
        Ok(())
    }
}

impl<S> Debug for TableLogic<S>
where
    S: TableStorage + Collection + Default,
    S::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.range_ids_values()).finish()
    }
}
