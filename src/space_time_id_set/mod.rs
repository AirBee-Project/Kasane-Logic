use std::collections::{BTreeMap, HashMap, HashSet};

use crate::bit_vec::BitVec;
pub mod single;

type Index = usize;
pub mod get_all;
pub mod insert;

#[derive(Debug)]
pub struct LayerInfo {
    //その階層が持つ実際のIDのIndex
    pub index: HashSet<Index>,

    //その階層の下にあるIDの個数
    pub count: usize,
}

#[derive(Debug)]
pub struct ReverseInfo {
    f: BitVec,
    x: BitVec,
    y: BitVec,
}

#[derive(Debug)]
pub struct SpaceTimeIdSet {
    //各次元の範囲を保存するためのBTreeMap
    f: BTreeMap<BitVec, LayerInfo>,
    x: BTreeMap<BitVec, LayerInfo>,
    y: BTreeMap<BitVec, LayerInfo>,
    index: usize,
    reverse: HashMap<Index, ReverseInfo>,
}
impl SpaceTimeIdSet {
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            index: 0,
            reverse: HashMap::new(),
        }
    }
}
