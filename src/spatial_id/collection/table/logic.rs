use crate::{
    BTreeMapTrait,
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
    pub fn open(table_storage: S) -> Self {
        Self(table_storage)
    }

    pub fn load() {}

    pub fn close(self) -> S {
        self.0
    }

    pub fn size(&self) -> usize {
        self.0.main().len()
    }

    pub fn insert<I: ToFlexId>(&mut self, target: &I) {
        todo!()
    }
}
