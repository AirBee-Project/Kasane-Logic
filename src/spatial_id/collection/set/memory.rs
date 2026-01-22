use std::{
    collections::{BTreeMap, HashMap}, // HashMapを追加
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use roaring::RoaringTreemap;

use crate::{
    KeyValueStore,
    spatial_id::{
        collection::{
            Collection, FlexIdRank, MAX_RECYCLE_CAPACITY,
            set::{SetStorage, logic::SetLogic},
        },
        flex_id::FlexId,
        segment::Segment,
    },
};

//===========================================
//ユーザーが実際に触るデフォルトの「Set」型
#[derive(Default)]
pub struct SetOnMemory(pub(crate) SetLogic<SetOnMemoryInner>);

impl Deref for SetOnMemory {
    type Target = SetLogic<SetOnMemoryInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SetOnMemory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Clone for SetOnMemory {
    fn clone(&self) -> Self {
        Self::load(&**self)
    }
}

impl SetOnMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load<S>(storage: &S) -> Self
    where
        S: SetStorage + Collection,
    {
        let main: HashMap<FlexIdRank, FlexId> =
            storage.main().iter().map(|(k, v)| (k, v.clone())).collect();

        let next_rank = storage.allocation_cursor();

        let copy_dim = |source: &S::Dimension| -> BTreeMap<Segment, RoaringTreemap> {
            source.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };

        let f = copy_dim(storage.f());
        let x = copy_dim(storage.x());
        let y = copy_dim(storage.y());

        let inner = SetOnMemoryInner {
            f,
            x,
            y,
            main,
            next_rank,
            recycled_ranks: storage.free_list(),
        };

        Self(SetLogic::open(inner))
    }

    pub fn into_inner(self) -> SetLogic<SetOnMemoryInner> {
        self.0
    }
}

impl Display for SetOnMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Debug for SetOnMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, PartialEq)]
pub struct SetOnMemoryInner {
    pub(crate) f: BTreeMap<Segment, RoaringTreemap>,
    pub(crate) x: BTreeMap<Segment, RoaringTreemap>,
    pub(crate) y: BTreeMap<Segment, RoaringTreemap>,

    pub(crate) main: HashMap<FlexIdRank, FlexId>,

    pub(crate) next_rank: u64,
    pub(crate) recycled_ranks: Vec<u64>,
}

impl Default for SetOnMemoryInner {
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

impl SetStorage for SetOnMemoryInner {}

impl Collection for SetOnMemoryInner {
    type Dimension = BTreeMap<Segment, RoaringTreemap>;
    type Main = HashMap<FlexIdRank, FlexId>;

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

    // 追加実装
    fn allocation_cursor(&self) -> u64 {
        self.next_rank
    }

    // 追加実装
    fn free_list(&self) -> Vec<u64> {
        self.recycled_ranks.clone()
    }
}
