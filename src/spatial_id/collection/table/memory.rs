use std::fmt::Debug;
use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Collection, FlexIdRank, MAX_RECYCLE_CAPACITY, ValueRank,
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

impl<V> TableOnMemory<V>
where
    V: Clone + PartialEq + Ord,
{
    pub fn new() -> Self {
        Self::default()
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
    main: BTreeMap<FlexIdRank, FlexId>,
    forward: BTreeMap<FlexIdRank, ValueRank>,
    dictionary: BTreeMap<ValueRank, V>,
    reserve: BTreeMap<V, ValueRank>,
    flex_id_next_rank: u64,
    flex_id_recycled_ranks: Vec<u64>,
    value_next_rank: u64,
    value_recycled_ranks: Vec<u64>,
}

impl<V> Default for TableOnMemoryInner<V> {
    fn default() -> Self {
        Self {
            f: Default::default(),
            x: Default::default(),
            y: Default::default(),
            main: Default::default(),
            forward: Default::default(),
            dictionary: Default::default(),
            reserve: Default::default(),
            flex_id_next_rank: 0,
            flex_id_recycled_ranks: vec![],
            value_next_rank: 0,
            value_recycled_ranks: vec![],
        }
    }
}

impl<V> TableStorage for TableOnMemoryInner<V>
where
    V: Clone + PartialEq + Ord,
{
    type Value = V;
    type Forward = BTreeMap<FlexIdRank, ValueRank>;
    type Dictionary = BTreeMap<ValueRank, V>;
    type Reverse = BTreeMap<V, ValueRank>;

    fn forward(&self) -> &Self::Forward {
        &self.forward
    }

    fn forward_mut(&mut self) -> &mut Self::Forward {
        &mut self.forward
    }

    fn dictionary(&self) -> &Self::Dictionary {
        &self.dictionary
    }

    fn dictionary_mut(&mut self) -> &mut Self::Dictionary {
        &mut self.dictionary
    }

    fn reverse(&self) -> &Self::Reverse {
        &self.reserve
    }

    fn reverse_mut(&mut self) -> &mut Self::Reverse {
        &mut self.reserve
    }

    fn fetch_value_rank(&mut self) -> u64 {
        self.value_next_rank
    }

    fn return_value_rank(&mut self, rank: u64) {
        if self.value_recycled_ranks.len() < MAX_RECYCLE_CAPACITY {
            self.value_recycled_ranks.push(rank);
        }
    }

    fn move_value_rank(&self) -> u64 {
        self.value_next_rank
    }

    fn move_value_rank_free_list(&self) -> Vec<u64> {
        self.value_recycled_ranks.clone()
    }
}

impl<V> Collection for TableOnMemoryInner<V>
where
    V: Clone + PartialEq + Ord,
{
    type Dimension = BTreeMap<Segment, RoaringTreemap>;
    type Main = BTreeMap<FlexIdRank, FlexId>;

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

    fn fetch_flex_rank(&mut self) -> u64 {
        if let Some(rank) = self.flex_id_recycled_ranks.pop() {
            return rank;
        }
        let rank = self.flex_id_next_rank;
        self.flex_id_next_rank += 1;
        rank
    }

    fn return_flex_rank(&mut self, rank: u64) {
        if self.flex_id_recycled_ranks.len() < MAX_RECYCLE_CAPACITY {
            self.flex_id_recycled_ranks.push(rank);
        }
    }

    fn move_flex_rank(&self) -> u64 {
        self.flex_id_next_rank
    }

    fn move_flex_rank_free_list(&self) -> Vec<u64> {
        self.flex_id_recycled_ranks.clone()
    }
}

impl<V> Display for TableOnMemory<V>
where
    V: Clone + PartialEq + Ord + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<V> Debug for TableOnMemory<V>
where
    V: Clone + PartialEq + Ord + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
