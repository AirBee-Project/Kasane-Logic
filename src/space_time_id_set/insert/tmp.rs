use std::collections::{BTreeMap, HashSet};
use std::ops::Bound::{Excluded, Included};

use crate::{
    space_time_id_set::{Index, LayerInfo, SpaceTimeIdSet},
    r#type::bit_vec::BitVec,
};

pub enum MainDimensionSelect {
    F,
    X,
    Y,
}

struct DimensionRefs<'a> {
    main: &'a BTreeMap<BitVec, LayerInfo>,
    others: [&'a BTreeMap<BitVec, LayerInfo>; 2],
}

impl SpaceTimeIdSet {
    /// 対応する次元をまとめて返す構造体

    /// メイン次元とその他の次元の参照を選択
    fn select_dimensions(&self, dim: &MainDimensionSelect) -> DimensionRefs<'_> {
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
                others: [&self.x, &self.f],
            },
        }
    }

    /// 指定された BitVec の上位層インデックスを集める
    fn collect_top_indices(map: &BTreeMap<BitVec, LayerInfo>, key: &BitVec) -> HashSet<Index> {
        let mut result = HashSet::new();
        for top in key.top_prefix() {
            if let Some(v) = map.get(&top) {
                result.extend(v.index.iter().copied());
            }
        }
        result
    }

    /// 他次元との組み合わせで uncheck_insert を呼ぶ
    fn insert_combinations(
        &mut self,
        dim_sel: &MainDimensionSelect,
        main: &BitVec,
        others: &[&Vec<(usize, BitVec)>; 2],
    ) {
        // 他次元が少なくとも2つの要素を持つかチェック
        if others.len() != 2 {
            return;
        }

        // 組み合わせを生成して挿入
        for (_, b1) in &*others[0] {
            for (_, b2) in &*others[1] {
                match dim_sel {
                    MainDimensionSelect::F => self.uncheck_insert(main, b1, b2),
                    MainDimensionSelect::X => self.uncheck_insert(b1, main, b2),
                    MainDimensionSelect::Y => self.uncheck_insert(b1, b2, main),
                }
            }
        }
    }

    /// 元の tmp 処理（簡潔化）
    pub fn tmp(
        &mut self,
        main_under_count: &usize,
        main: &BitVec,
        main_encoded: &mut Vec<(usize, BitVec)>,
        main_index: &usize,
        other_encoded: &[&Vec<(usize, BitVec)>; 2],
        main_sel: MainDimensionSelect,
    ) {
        let main_top;
        {
            let dims = self.select_dimensions(&main_sel);

            // 上位を収集
            main_top = Self::collect_top_indices(dims.main, main);

            // 上位も下位も0の場合の処理
            if main_top.is_empty() && *main_under_count == 0 {
                if *main_index < main_encoded.len() {
                    let _removed = main_encoded.remove(*main_index);
                    self.insert_combinations(&main_sel, main, other_encoded);
                }
            }
        }

        //下位の範囲を検索
        let mut main_under = HashSet::new();
        {
            let dims = self.select_dimensions(&main_sel);
            for (_, layerinfo) in dims
                .main
                .range((Included(main), Excluded(&main.under_prefix())))
            {
                main_under.extend(layerinfo.index.clone());
            }
        }

        //逆引きをして範囲を照合

        //main_topから検索

        //全てが上位の場合→Return

        //main_topのみが上位で、残りの2つが上位の場合

        //main_topと1つが上位で、残りの1つが下位の場合

        //------------------------------------------

        //main_underを検索

        //全てが下位の場合→そのIDをdelete

        //main_underのみが下位で、残りの2つが上位の場合

        //main_topと1つが下位で、残りの1つが上位の場合
    }
}
