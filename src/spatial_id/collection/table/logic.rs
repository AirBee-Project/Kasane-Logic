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

    ///重複確認なく挿入を行う
    /// 結合の最適化を行わないとEqなどが正常に動作しなくなる
    /// 結合最適化を行ったものを入れないと、ロジックが壊れる
    /// もしくは、明らかに結合不能なIDなど
    pub unsafe fn insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: S::Value) {
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

    pub unsafe fn join_insert_unchecked<I: ToFlexId>(&mut self, target: &I, value: S::Value) {
        for flex_id in target.flex_ids() {
            // クロージャ定義: 値の一致確認ロジック
            // (Forward -> Dictionary を引いて値の実体を比較)
            let is_same_value = |storage: &S, rank: &FlexIdRank| -> bool {
                storage
                    .forward()
                    .get(rank)
                    .and_then(|val_rank| storage.dictionary().get(&val_rank))
                    .map_or(false, |v| v == value)
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
                        unsafe { self.join_insert_unchecked(&parent, value.clone()) };
                        continue;
                    }
                }
            }

            if let Some(sibling_rank) = self.0.get_x_sibling_flex_id(&flex_id) {
                if is_same_value(&self.0, &sibling_rank) {
                    let sibling_id = self.0.get_flex_id(sibling_rank).unwrap().clone();
                    if let Some(parent) = sibling_id.x_parent() {
                        remove_sibling(self, sibling_rank);
                        unsafe { self.join_insert_unchecked(&parent, value.clone()) };
                        continue;
                    }
                }
            }

            if let Some(sibling_rank) = self.0.get_y_sibling_flex_id(&flex_id) {
                if is_same_value(&self.0, &sibling_rank) {
                    let sibling_id = self.0.get_flex_id(sibling_rank).unwrap().clone();
                    if let Some(parent) = sibling_id.y_parent() {
                        remove_sibling(self, sibling_rank);
                        unsafe { self.join_insert_unchecked(&parent, value.clone()) };
                        continue;
                    }
                }
            }

            let new_rank = self.0.fetch_flex_rank();
            let mut main_batch = Batch::new();
            main_batch.put(new_rank, flex_id);
            self.0.main_mut().apply_batch(main_batch);
            self.0.insert_value(value.clone(), vec![new_rank]);
        }
    }
}
