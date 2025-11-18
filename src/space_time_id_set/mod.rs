use std::collections::{BTreeMap, HashMap, HashSet};

use crate::bit_vec::BitVec;
pub mod single;

type Index = usize;
pub mod get_all;
pub mod insert;

/// 階層情報を保持する構造体
///
/// 各階層が持つIDのインデックスと、その階層の下にあるIDの個数を管理する。
#[derive(Debug)]
pub struct LayerInfo {
    //その階層が持つ実際のIDのIndex
    pub index: HashSet<Index>,

    //その階層の下にあるIDの個数
    pub count: usize,
}

/// 逆引き情報を保持する構造体
///
/// インデックスから各次元のBitVecを取得するために使用する。
#[derive(Hash, Eq, PartialEq, Debug)]
pub struct ReverseInfo {
    pub f: BitVec,
    pub x: BitVec,
    pub y: BitVec,
}

/// 時空間IDの集合を効率的に管理する構造体
///
/// 複数の時空間IDを階層的に管理し、重複や包含関係を解決しながら挿入する。
/// 各次元（F, X, Y）をBTreeMapで管理し、高速な範囲検索を可能にする。
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
    /// 新しい空の時空間IDセットを作成する
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

impl Default for SpaceTimeIdSet {
    fn default() -> Self {
        Self::new()
    }
}
