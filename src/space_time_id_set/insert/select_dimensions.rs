use std::collections::BTreeMap;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{LayerInfo, SpaceTimeIdSet, insert::insert_main_dim::MainDimensionSelect},
};

pub struct DimensionRefs<'a> {
    pub main: &'a BTreeMap<BitVec, LayerInfo>,
    pub others: [&'a BTreeMap<BitVec, LayerInfo>; 2],
}

impl SpaceTimeIdSet {
    /// メイン次元とその他の次元の参照を選択
    pub fn select_dimensions(&self, dim: &MainDimensionSelect) -> DimensionRefs<'_> {
        match dim {
            MainDimensionSelect::F => DimensionRefs {
                main: &self.f,
                others: [&self.x, &self.y],
            },
            MainDimensionSelect::X => DimensionRefs {
                main: &self.x,
                others: [&self.f, &self.y],
            },
            MainDimensionSelect::Y => DimensionRefs {
                main: &self.y,
                others: [&self.f, &self.x],
            },
        }
    }
}
