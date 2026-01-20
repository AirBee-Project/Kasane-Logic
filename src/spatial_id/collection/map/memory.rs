use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::spatial_id::{
    collection::{
        Rank,
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
}

impl<V> Default for MapOnMemoryInner<V> {
    fn default() -> Self {
        Self {
            f: Default::default(),
            x: Default::default(),
            y: Default::default(),
            main: Default::default(),
        }
    }
}

impl<V> MapStorage for MapOnMemoryInner<V>
where
    V: Clone + PartialEq,
{
    type Value = V;

    type Main = BTreeMap<Rank, (FlexId, V)>;
    type Dimension = BTreeMap<Segment, RoaringTreemap>;

    fn main(&self) -> &Self::Main {
        todo!()
    }

    fn main_mut(&mut self) -> &mut Self::Main {
        todo!()
    }

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
}
