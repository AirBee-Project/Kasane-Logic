use std::fmt::{self, Display};

use crate::{
    KeyValueStore, RangeId, SingleId,
    spatial_id::{
        ToFlexId,
        collection::{
            Collection, FlexIdRank,
            table::{TableStorage, memory::TableOnMemory},
        },
        flex_id::FlexId,
    },
    storage::Batch,
};
use std::fmt::Debug;

#[derive(Default)]
pub struct TableLogic<S: TableStorage + Collection>(S);

impl<S> TableLogic<S>
where
    S: TableStorage + Collection + Default,
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
        Box::new(self.0.main().iter().filter_map(|(rank, flex_id)| {
            // 1. FlexIdRank -> ValueRank (Forward)
            let val_rank = self.0.forward().get(&rank)?;

            // 2. ValueRank -> Value (Dictionary)
            let value = self.0.dictionary().get(&val_rank)?;

            // FlexIdは参照で来る場合もあるのでClone、Valueは所有権として返す
            Some((flex_id.clone(), value))
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

    pub fn insert<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        for new_id in target.flex_ids() {
            let collisions = self.0.resolve_collisions(&new_id);
            for (removed_rank, fragments) in collisions {
                if let Some(old_val_rank) = self.0.forward().get(&removed_rank) {
                    if let Some(old_value) = self.0.dictionary().get(&old_val_rank) {
                        for frag in fragments {
                            unsafe { self.join_insert_unchecked(&frag, &old_value) };
                        }
                    }
                    let mut batch = Batch::new();
                    batch.delete(removed_rank);
                    self.0.forward_mut().apply_batch(batch);
                }
            }
            unsafe { self.join_insert_unchecked(&new_id, value) };
        }
    }

    pub fn get<I: ToFlexId>(&mut self, target: &I) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                let value_rank = self.0.forward().get(&related_rank).unwrap();
                let value = self.0.dictionary().get(&value_rank).unwrap();
                result.insert(&flex_id.intersection(&related_id).unwrap(), &value);
            }
        }
        result
    }

    pub fn remove<I: ToFlexId>(&mut self, target: &I) -> TableOnMemory<S::Value> {
        let mut result = TableOnMemory::default();
        for flex_id in target.flex_ids() {
            for related_rank in self.0.related(&flex_id) {
                let related_id = self.0.get_flex_id(related_rank).unwrap();
                let value_rank = self.0.forward().get(&related_rank).unwrap();
                let value = self.0.dictionary().get(&value_rank).unwrap();
                for removed_flex_id in flex_id.difference(&related_id) {
                    result.insert(&removed_flex_id, &value);
                }
            }
        }
        result
    }

    ///重複確認なく挿入を行う
    /// 結合の最適化を行わないとEqなどが正常に動作しなくなる
    /// 結合最適化を行ったものを入れないと、ロジックが壊れる
    /// もしくは、明らかに結合不能なIDなど
    pub unsafe fn insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        let mut flex_id_ranks = Vec::new();
        let mut main_batch = Batch::new();
        for flex_id in target.flex_ids() {
            let rank = self.0.fetch_flex_rank();
            main_batch.put(rank, flex_id);
            flex_id_ranks.push(rank);
        }
        self.0.main_mut().apply_batch(main_batch);
        self.0.insert_value(value, flex_id_ranks);
    }

    pub unsafe fn join_insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: &S::Value) {
        for flex_id in target.flex_ids() {
            // (Forward -> Dictionary を引いて値の実体を比較)
            let is_same_value = |storage: &S, rank: &FlexIdRank| -> bool {
                storage
                    .forward()
                    .get(rank)
                    .and_then(|val_rank| storage.dictionary().get(&val_rank))
                    .map_or(false, |v| v == value.clone())
            };

            let remove_sibling = |me: &mut Self, rank: FlexIdRank| {
                me.0.remove_flex_id(rank);
                let mut batch = Batch::new();
                batch.delete(rank);
                me.0.forward_mut().apply_batch(batch);
            };

            if let Some(sibling_rank) = self.0.get_f_sibling_flex_id(&flex_id) {
                if is_same_value(&self.0, &sibling_rank) {
                    // ID実体を取得して親を計算 (remove前に取得必須)
                    let sibling_id = self.0.get_flex_id(sibling_rank).unwrap().clone();
                    if let Some(parent) = sibling_id.f_parent() {
                        remove_sibling(self, sibling_rank);
                        unsafe { self.join_insert_unchecked(&parent, value) };
                        continue;
                    }
                }
            }

            if let Some(sibling_rank) = self.0.get_x_sibling_flex_id(&flex_id) {
                if is_same_value(&self.0, &sibling_rank) {
                    let sibling_id = self.0.get_flex_id(sibling_rank).unwrap().clone();
                    if let Some(parent) = sibling_id.x_parent() {
                        remove_sibling(self, sibling_rank);
                        unsafe { self.join_insert_unchecked(&parent, value) };
                        continue;
                    }
                }
            }

            if let Some(sibling_rank) = self.0.get_y_sibling_flex_id(&flex_id) {
                if is_same_value(&self.0, &sibling_rank) {
                    let sibling_id = self.0.get_flex_id(sibling_rank).unwrap().clone();
                    if let Some(parent) = sibling_id.y_parent() {
                        remove_sibling(self, sibling_rank);
                        unsafe { self.join_insert_unchecked(&parent, value) };
                        continue;
                    }
                }
            }

            let new_rank = self.0.fetch_flex_rank();
            let mut main_batch = Batch::new();
            main_batch.put(new_rank, flex_id);
            self.0.main_mut().apply_batch(main_batch);
            self.0.insert_value(value, vec![new_rank]);
        }
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
