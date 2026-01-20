use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Collection, MAX_RECYCLE_CAPACITY, Rank,
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
    index: BTreeMap<V, RoaringTreemap>,
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
            index: Default::default(),
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
    type Index = BTreeMap<V, RoaringTreemap>;

    fn index(&self) -> &Self::Index {
        &self.index
    }

    fn index_mut(&mut self) -> &mut Self::Index {
        &mut self.index
    }
}

impl<V> Collection for TableOnMemoryInner<V>
where
    V: Clone + PartialEq + Ord,
{
    type Dimension = BTreeMap<Segment, RoaringTreemap>;
    type Value = V;
    type Main = BTreeMap<Rank, (FlexId, Self::Value)>;

    fn main(&self) -> &Self::Main {
        &self.main
    }

    fn main_mut(&mut self) -> &mut Self::Main {
        &mut self.main
    }

    fn f(&self) -> &Self::Dimension {
        &self.f
    }

    fn f_mut(&mut self) -> &mut Self::Dimension {
        &mut self.f
    }

    fn x(&self) -> &Self::Dimension {
        &self.x
    }

    fn x_mut(&mut self) -> &mut Self::Dimension {
        &mut self.x
    }

    fn y(&self) -> &Self::Dimension {
        &self.y
    }

    fn y_mut(&mut self) -> &mut Self::Dimension {
        &mut self.y
    }

    fn fetch_rank(&mut self) -> u64 {
        if let Some(rank) = self.recycled_ranks.pop() {
            return rank;
        }
        let rank = self.next_rank;
        self.next_rank += 1;
        rank
    }

    fn return_rank(&mut self, rank: u64) {
        if self.recycled_ranks.len() < MAX_RECYCLE_CAPACITY {
            self.recycled_ranks.push(rank);
        }
    }
}
