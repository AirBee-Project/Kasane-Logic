use crate::{
    KeyValueStore,
    spatial_id::{
        ToFlexId,
        collection::{
            Collection,
            table::{TableStorage, memory::TableOnMemory},
        },
    },
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

    pub fn insert<I: ToFlexId>(&mut self, target: &I, value: S::Value) {
        todo!()
    }
}
