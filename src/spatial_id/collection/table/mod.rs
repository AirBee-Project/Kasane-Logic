use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use roaring::RoaringTreemap;

use crate::{FlexId, FlexIdRank, Segment, spatial_id::FlexIds};
pub type ValueRank = u64;

pub struct TableOnMemory<V> {
    f: BTreeMap<Segment, RoaringTreemap>,
    x: BTreeMap<Segment, RoaringTreemap>,
    y: BTreeMap<Segment, RoaringTreemap>,
    main: HashMap<FlexIdRank, (FlexId, ValueRank)>,
    next_rank: u64,
    recycle_rank: Vec<u64>,

    //Table特有の要素
    dictionary: BTreeMap<V, RoaringTreemap>,
    reverse: HashMap<ValueRank, V>,
    value_next_rank: u64,
    value_recycle_rank: Vec<u64>,
}

impl<V> TableOnMemory<V> {
    const RECYCLE_RANK_MAX: usize = 1024;

    ///初期化する
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            main: HashMap::new(),
            next_rank: 0,
            recycle_rank: Vec::new(),
            dictionary: BTreeMap::new(),
            reverse: HashMap::new(),
            value_next_rank: 0,
            value_recycle_rank: Vec::new(),
        }
    }

    ///値を挿入する
    pub fn insert<T: FlexIds>(&mut self, target: &T, value: &V) {
        //まずは値について考えたほうが良いよね
    }

    ///新しいRankを予約するためのメソット
    fn fetch_rank(&mut self) -> FlexIdRank {
        match self.recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.next_rank;
                self.next_rank = self.next_rank + 1;
                result
            }
        }
    }

    ///Rankをreturnするためのメソット
    fn return_rank(&mut self, rank: u64) {
        if self.recycle_rank.len() < Self::RECYCLE_RANK_MAX {
            self.recycle_rank.push(rank);
        }
    }

    ///新しいRankを予約するためのメソット
    fn fetch_value_rank(&mut self) -> ValueRank {
        match self.value_recycle_rank.pop() {
            Some(v) => v,
            None => {
                let result = self.value_next_rank;
                self.value_next_rank = self.value_next_rank + 1;
                result
            }
        }
    }

    ///Rankをreturnするためのメソット
    fn return_value_rank(&mut self, rank: u64) {
        if self.value_recycle_rank.len() < Self::RECYCLE_RANK_MAX {
            self.value_recycle_rank.push(rank);
        }
    }
}
