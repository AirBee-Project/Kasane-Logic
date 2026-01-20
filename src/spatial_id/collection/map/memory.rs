use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Collection, MAX_RECYCLE_CAPACITY, Rank,
        map::{MapStorage, logic::MapLogic},
    },
    flex_id::FlexId,
    segment::Segment,
};

pub struct MapOnMemory<V>(MapLogic<MapOnMemoryInner<V>>)
where
    V: Clone + PartialEq;

impl<V> Default for MapOnMemory<V>
where
    V: Clone + PartialEq,
{
    fn default() -> Self {
        Self(MapLogic::open(MapOnMemoryInner::default()))
    }
}

impl<V> Deref for MapOnMemory<V>
where
    V: Clone + PartialEq,
{
    type Target = MapLogic<MapOnMemoryInner<V>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for MapOnMemory<V>
where
    V: Clone + PartialEq,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct MapOnMemoryInner<V> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: BTreeMap<Rank, (FlexId, V)>,
    next_rank: u64,
    recycled_ranks: Vec<u64>,
}

impl<V> Default for MapOnMemoryInner<V> {
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

impl<V> MapStorage for MapOnMemoryInner<V>
where
    V: Clone + PartialEq,
{
    type Value = V;

    type Main = BTreeMap<Rank, (FlexId, V)>;

    fn main(&self) -> &Self::Main {
        todo!()
    }

    fn main_mut(&mut self) -> &mut Self::Main {
        todo!()
    }
}

impl<V> Collection for MapOnMemoryInner<V> {
    type Dimension = BTreeMap<Segment, RoaringTreemap>;

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
