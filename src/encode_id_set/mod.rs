use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{bit_vec::BitVec, encode_id::EncodeID};

///Set内部におけるIDの一意な番号
type Index = usize;

pub mod difference;
pub mod get;
pub mod insert;
pub mod intersection;
pub mod iterator;
pub mod remove;
pub mod union;
pub mod utils;

/// 階層ごとの情報を保持する構造体
#[derive(Debug, Clone)]
pub struct LayerInfo {
    //その階層が持つ実際のIDのIndex
    pub index: HashSet<Index>,

    //その階層の下にあるIDの個数
    pub count: usize,
}

/// 時空間IDの集合を効率的に管理するデータ構造
///
/// 重複する範囲を自動的に統合し、階層構造を用いて効率的に格納する。
/// 公開APIは`insert`と`get_all`のみ。
#[derive(Debug, Clone)]
pub struct EncodeIDSet {
    //各次元の範囲を保存するためのBTreeMap
    f: BTreeMap<BitVec, LayerInfo>,
    x: BTreeMap<BitVec, LayerInfo>,
    y: BTreeMap<BitVec, LayerInfo>,
    t: BTreeMap<BitVec, LayerInfo>,
    index: usize,
    reverse: HashMap<Index, EncodeID>,
}
impl EncodeIDSet {
    /// 新しい空の時空間ID集合を作成
    pub fn new() -> Self {
        Self {
            f: BTreeMap::new(),
            x: BTreeMap::new(),
            y: BTreeMap::new(),
            t: BTreeMap::new(),
            index: 0,
            reverse: HashMap::new(),
        }
    }
}
