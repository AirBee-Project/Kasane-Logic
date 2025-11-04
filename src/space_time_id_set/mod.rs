use crate::{space_time_id::SpaceTimeId, r#type::bit_vec::BitVec};
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
};
pub mod single;

type Index = usize;
pub mod insert;

pub struct LayerInfo {
    //その階層が持つ実際のIDのIndex
    pub index: HashSet<Index>,

    //その階層の下にあるIDの個数
    pub count: usize,
}

pub struct SpaceTimeIdSet {
    //各次元の範囲を保存するためのBTreeMap
    f: BTreeMap<BitVec, LayerInfo>,
    x: BTreeMap<BitVec, LayerInfo>,
    y: BTreeMap<BitVec, LayerInfo>,
}
impl SpaceTimeIdSet {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
        }
    }
}
