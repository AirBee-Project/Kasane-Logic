use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Collection, Rank,
        table::{TableStorage, logic::TableLogic},
    },
    flex_id::FlexId,
    segment::Segment,
};

pub struct TableOnMemory<V>(TableLogic<TableOnMemoryInner<V>>)
where
    V: Clone + PartialEq + Ord;

impl<V> Default for TableOnMemory<V>
where
    V: Clone + PartialEq + Ord,
{
    fn default() -> Self {
        Self(TableLogic::open(TableOnMemoryInner::default()))
    }
}

impl<V> Deref for TableOnMemory<V>
where
    V: Clone + PartialEq + Ord,
{
    type Target = TableLogic<TableOnMemoryInner<V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for TableOnMemory<V>
where
    V: Clone + PartialEq + Ord,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct TableOnMemoryInner<V> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<Rank, (FlexId, V)>,
    next_rank: u64,
    recycled_ranks: Vec<u64>,
}

impl<V> Default for TableOnMemoryInner<V> {
    fn default() -> Self {
        Self {
            f: Default::default(),
            x: Default::default(),
            y: Default::default(),
            main: Default::default(),
            next_rank: 0,
            recycled_ranks: vec![],
        }
    }
}

impl<V> TableStorage for TableOnMemoryInner<V>
where
    V: Clone + PartialEq + Ord,
{
    type Value = V;
    type Main = BTreeMap<Rank, (FlexId, V)>;
    type Index = BTreeMap<V, RoaringTreemap>;

    fn main(&self) -> &Self::Main {
        todo!()
    }

    fn main_mut(&mut self) -> &mut Self::Main {
        todo!()
    }

    fn index(&self) -> Self::Index {
        todo!()
    }

    fn index_mut(&mut self) -> Self::Index {
        todo!()
    }
}

impl<V> Collection for TableOnMemoryInner<V> {
    type Dimension = BTreeMap<Segment, RoaringTreemap>;

    fn f(&self) -> &Self::Dimension {
        todo!()
    }

    fn f_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn x(&self) -> &Self::Dimension {
        todo!()
    }

    fn x_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn y(&self) -> &Self::Dimension {
        todo!()
    }

    fn y_mut(&mut self) -> &mut Self::Dimension {
        todo!()
    }

    fn fetch_rank(&mut self) -> u64 {
        todo!()
    }

    fn return_rank(&mut self, rank: u64) {
        todo!()
    }
}
