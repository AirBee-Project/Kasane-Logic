use std::collections::BTreeMap;

use crate::{
    bit_vec::HierarchicalKey,
    space_time_id_set::{LayerInfo, SpaceTimeIdSet},
};

pub enum DimensionSelect {
    F,
    X,
    Y,
}

impl DimensionSelect {
    pub fn a(&self) -> DimensionSelect {
        match self {
            DimensionSelect::F => DimensionSelect::X,
            DimensionSelect::X => DimensionSelect::F,
            DimensionSelect::Y => DimensionSelect::F,
        }
    }

    pub fn b(&self) -> DimensionSelect {
        match self {
            DimensionSelect::F => DimensionSelect::Y,
            DimensionSelect::X => DimensionSelect::Y,
            DimensionSelect::Y => DimensionSelect::X,
        }
    }
}

// BTreeMap参照系（BTreeを型名に含める）
pub struct DimensionBTreeRefs<'a> {
    pub main: &'a BTreeMap<HierarchicalKey, LayerInfo>,
    pub a: &'a BTreeMap<HierarchicalKey, LayerInfo>,
    pub b: &'a BTreeMap<HierarchicalKey, LayerInfo>,
}

pub struct DimensionBTreeMutRefs<'a> {
    pub main: &'a mut BTreeMap<HierarchicalKey, LayerInfo>,
    pub a: &'a mut BTreeMap<HierarchicalKey, LayerInfo>,
    pub b: &'a mut BTreeMap<HierarchicalKey, LayerInfo>,
}

pub struct DimensionBTreeRevRefs<'a> {
    pub main: &'a BTreeMap<HierarchicalKey, LayerInfo>,
    pub a: &'a BTreeMap<HierarchicalKey, LayerInfo>,
    pub b: &'a BTreeMap<HierarchicalKey, LayerInfo>,
}

// 汎用型マッピング型
pub struct MapDimsRefs<'a, T> {
    pub f: &'a T,
    pub x: &'a T,
    pub y: &'a T,
}

pub struct MapDimsMutRefs<'a, T> {
    pub f: &'a mut T,
    pub x: &'a mut T,
    pub y: &'a mut T,
}

impl SpaceTimeIdSet {
    // BTreeMap参照
    pub fn dims_btree(&self, main_dim_select: &DimensionSelect) -> DimensionBTreeRefs<'_> {
        match main_dim_select {
            DimensionSelect::F => DimensionBTreeRefs {
                main: &self.f,
                a: &self.x,
                b: &self.y,
            },
            DimensionSelect::X => DimensionBTreeRefs {
                main: &self.x,
                a: &self.f,
                b: &self.y,
            },
            DimensionSelect::Y => DimensionBTreeRefs {
                main: &self.y,
                a: &self.f,
                b: &self.x,
            },
        }
    }

    // BTreeMap可変参照
    pub fn dims_btree_mut(
        &mut self,
        main_dim_select: &DimensionSelect,
    ) -> DimensionBTreeMutRefs<'_> {
        match main_dim_select {
            DimensionSelect::F => DimensionBTreeMutRefs {
                main: &mut self.f,
                a: &mut self.x,
                b: &mut self.y,
            },
            DimensionSelect::X => DimensionBTreeMutRefs {
                main: &mut self.x,
                a: &mut self.f,
                b: &mut self.y,
            },
            DimensionSelect::Y => DimensionBTreeMutRefs {
                main: &mut self.y,
                a: &mut self.f,
                b: &mut self.x,
            },
        }
    }

    // 汎用型Tのマッピング
    pub fn map_dims<'a, T>(
        main: &'a T,
        a: &'a T,
        b: &'a T,
        main_dim_select: &DimensionSelect,
    ) -> MapDimsRefs<'a, T> {
        match main_dim_select {
            DimensionSelect::F => MapDimsRefs {
                f: main,
                x: a,
                y: b,
            },
            DimensionSelect::X => MapDimsRefs {
                f: a,
                x: main,
                y: b,
            },
            DimensionSelect::Y => MapDimsRefs {
                f: a,
                x: b,
                y: main,
            },
        }
    }

    pub fn map_dims_mut<'a, T>(
        main: &'a mut T,
        a: &'a mut T,
        b: &'a mut T,
        main_dim_select: &DimensionSelect,
    ) -> MapDimsMutRefs<'a, T> {
        match main_dim_select {
            DimensionSelect::F => MapDimsMutRefs {
                f: main,
                x: a,
                y: b,
            },
            DimensionSelect::X => MapDimsMutRefs {
                f: a,
                x: main,
                y: b,
            },
            DimensionSelect::Y => MapDimsMutRefs {
                f: a,
                x: b,
                y: main,
            },
        }
    }
}
