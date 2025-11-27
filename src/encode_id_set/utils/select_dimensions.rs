use std::collections::BTreeMap;

use crate::{
    bit_vec::BitVec,
    encode_id_set::{EncodeIDSet, LayerInfo},
};

#[derive(Debug)]
pub enum DimensionSelect {
    F,
    X,
    Y,
    T,
}

impl DimensionSelect {
    pub fn a(&self) -> DimensionSelect {
        match self {
            DimensionSelect::F => DimensionSelect::X,
            DimensionSelect::X => DimensionSelect::F,
            DimensionSelect::Y => DimensionSelect::F,
            DimensionSelect::T => DimensionSelect::F,
        }
    }

    pub fn b(&self) -> DimensionSelect {
        match self {
            DimensionSelect::F => DimensionSelect::Y,
            DimensionSelect::X => DimensionSelect::Y,
            DimensionSelect::Y => DimensionSelect::X,
            DimensionSelect::T => DimensionSelect::X,
        }
    }

    pub fn c(&self) -> DimensionSelect {
        match self {
            DimensionSelect::F => DimensionSelect::T,
            DimensionSelect::X => DimensionSelect::T,
            DimensionSelect::Y => DimensionSelect::T,
            DimensionSelect::T => DimensionSelect::Y,
        }
    }
}

// BTreeMap参照系（BTreeを型名に含める）
pub struct DimensionBTreeRefs<'a> {
    pub main: &'a BTreeMap<BitVec, LayerInfo>,
    pub a: &'a BTreeMap<BitVec, LayerInfo>,
    pub b: &'a BTreeMap<BitVec, LayerInfo>,
    pub c: &'a BTreeMap<BitVec, LayerInfo>,
}

pub struct DimensionBTreeMutRefs<'a> {
    pub main: &'a mut BTreeMap<BitVec, LayerInfo>,
    pub a: &'a mut BTreeMap<BitVec, LayerInfo>,
    pub b: &'a mut BTreeMap<BitVec, LayerInfo>,
    pub c: &'a mut BTreeMap<BitVec, LayerInfo>,
}

pub struct DimensionBTreeRevRefs<'a> {
    pub main: &'a BTreeMap<BitVec, LayerInfo>,
    pub a: &'a BTreeMap<BitVec, LayerInfo>,
    pub b: &'a BTreeMap<BitVec, LayerInfo>,
    pub c: &'a BTreeMap<BitVec, LayerInfo>,
}

// 汎用型マッピング型
pub struct MapDimsRefs<'a, T> {
    pub f: &'a T,
    pub x: &'a T,
    pub y: &'a T,
    pub t: &'a T,
}

pub struct MapDimsMutRefs<'a, T> {
    pub f: &'a mut T,
    pub x: &'a mut T,
    pub y: &'a mut T,
    pub t: &'a mut T,
}

impl EncodeIDSet {
    // BTreeMap参照
    pub fn dims_btree(&self, main_dim: &DimensionSelect) -> DimensionBTreeRefs<'_> {
        match main_dim {
            DimensionSelect::F => DimensionBTreeRefs {
                main: &self.f,
                a: &self.x,
                b: &self.y,
                c: &self.t,
            },
            DimensionSelect::X => DimensionBTreeRefs {
                main: &self.x,
                a: &self.f,
                b: &self.y,
                c: &self.t,
            },
            DimensionSelect::Y => DimensionBTreeRefs {
                main: &self.y,
                a: &self.f,
                b: &self.x,
                c: &self.t,
            },
            DimensionSelect::T => DimensionBTreeRefs {
                main: &self.t,
                a: &self.f,
                b: &self.x,
                c: &self.y,
            },
        }
    }

    // BTreeMap可変参照
    pub fn dims_btree_mut(&mut self, main_dim: &DimensionSelect) -> DimensionBTreeMutRefs<'_> {
        match main_dim {
            DimensionSelect::F => DimensionBTreeMutRefs {
                main: &mut self.f,
                a: &mut self.x,
                b: &mut self.y,
                c: &mut self.t,
            },
            DimensionSelect::X => DimensionBTreeMutRefs {
                main: &mut self.x,
                a: &mut self.f,
                b: &mut self.y,
                c: &mut self.t,
            },
            DimensionSelect::Y => DimensionBTreeMutRefs {
                main: &mut self.y,
                a: &mut self.f,
                b: &mut self.x,
                c: &mut self.t,
            },
            DimensionSelect::T => DimensionBTreeMutRefs {
                main: &mut self.t,
                a: &mut self.f,
                b: &mut self.x,
                c: &mut self.y,
            },
        }
    }

    // 汎用型Tのマッピング
    pub fn map_dims<'a, T>(
        main: &'a T,
        a: &'a T,
        b: &'a T,
        c: &'a T,
        main_dim: &DimensionSelect,
    ) -> MapDimsRefs<'a, T> {
        match main_dim {
            DimensionSelect::F => MapDimsRefs {
                f: main,
                x: a,
                y: b,
                t: c,
            },
            DimensionSelect::X => MapDimsRefs {
                f: a,
                x: main,
                y: b,
                t: c,
            },
            DimensionSelect::Y => MapDimsRefs {
                f: a,
                x: b,
                y: main,
                t: c,
            },
            DimensionSelect::T => MapDimsRefs {
                f: a,
                x: b,
                y: c,
                t: main,
            },
        }
    }

    pub fn map_dims_mut<'a, T>(
        main: &'a mut T,
        a: &'a mut T,
        b: &'a mut T,
        c: &'a mut T,
        main_dim: &DimensionSelect,
    ) -> MapDimsMutRefs<'a, T> {
        match main_dim {
            DimensionSelect::F => MapDimsMutRefs {
                f: main,
                x: a,
                y: b,
                t: c,
            },
            DimensionSelect::X => MapDimsMutRefs {
                f: a,
                x: main,
                y: b,
                t: c,
            },
            DimensionSelect::Y => MapDimsMutRefs {
                f: a,
                x: b,
                y: main,
                t: c,
            },
            DimensionSelect::T => MapDimsMutRefs {
                f: a,
                x: b,
                y: c,
                t: main,
            },
        }
    }
}
