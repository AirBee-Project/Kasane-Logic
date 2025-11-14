use itertools::iproduct;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{ReverseInfo, SpaceTimeIdSet, insert::check_relation::Relation},
};

#[derive(Clone, Copy, Debug)]
pub enum MainDimensionSelect {
    F,
    X,
    Y,
}

impl MainDimensionSelect {
    pub fn as_index(&self) -> usize {
        match self {
            MainDimensionSelect::F => 0,
            MainDimensionSelect::X => 1,
            MainDimensionSelect::Y => 2,
        }
    }
}

impl SpaceTimeIdSet {
    /// 代表次元×他の次元を挿入処理する
    pub fn insert_main_dim(
        &mut self,
        main_bit: &BitVec,
        main_index: &usize,
        main_under_count: &usize,
        main_encoded: &mut Vec<(usize, BitVec)>,
        other_encoded: &[&Vec<(usize, BitVec)>; 2],
        main_dim_select: MainDimensionSelect,
    ) {
        //代表次元における上位範囲を収拾する
        let main_top = Self::collect_top(&self, main_bit, &main_dim_select);

        //代表次元において、上位も下位も存在しなかった場合は無条件に挿入
        if main_top.is_empty() && *main_under_count == 0 {
            //挿入
            for ((_, a_bit), (_, b_bit)) in iproduct!(other_encoded[0], other_encoded[1]) {
                match main_dim_select {
                    MainDimensionSelect::F => self.uncheck_insert(main_bit, a_bit, b_bit),
                    MainDimensionSelect::X => self.uncheck_insert(a_bit, main_bit, b_bit),
                    MainDimensionSelect::Y => self.uncheck_insert(a_bit, b_bit, main_bit),
                };

                //代表次元を元の要素から削除
                let _removed = main_encoded.remove(*main_index);
                return;
            }
        }

        //代表次元において下位の範囲を収拾
        let main_under = self.collect_under(main_bit, &main_dim_select);

        //逆引き
        let mut top_reverse = vec![];
        for top_index in &main_top {
            top_reverse.push(self.reverse.get(&top_index).unwrap());
        }

        //逆引き
        let mut under_reverse = vec![];
        for top_index in &main_under {
            under_reverse.push(self.reverse.get(&top_index).unwrap());
        }

        let a_dim_select: MainDimensionSelect;
        let b_dim_select: MainDimensionSelect;

        match main_dim_select {
            MainDimensionSelect::F => {
                a_dim_select = MainDimensionSelect::X;
                b_dim_select = MainDimensionSelect::Y;
            }
            MainDimensionSelect::X => {
                a_dim_select = MainDimensionSelect::F;
                b_dim_select = MainDimensionSelect::Y;
            }
            MainDimensionSelect::Y => {
                a_dim_select = MainDimensionSelect::F;
                b_dim_select = MainDimensionSelect::X;
            }
        }

        //軸ごとに関係を見極める              MainTop         MainUnder
        let mut a_relations: Vec<Option<(Vec<Relation>, Vec<Relation>)>> = Vec::new();
        //軸ごとに関係を見極める              MainTop         MainUnder
        let mut b_relations: Vec<Option<(Vec<Relation>, Vec<Relation>)>> = Vec::new();

        //Aについて収拾する
        for (_, a_dim) in other_encoded[0] {
            a_relations.push(Self::collect_other_dimension(
                a_dim,
                a_dim_select,
                &top_reverse,
                &under_reverse,
            ));
        }

        //Bについて収拾する
        for (_, b_dim) in other_encoded[1] {
            b_relations.push(Self::collect_other_dimension(
                b_dim,
                b_dim_select,
                &top_reverse,
                &under_reverse,
            ));
        }

        for (a, b) in iproduct!(a_relations, b_relations) {
            let a_relations = match a {
                Some(v) => v,
                None => {
                    //無条件挿入
                    continue;
                }
            };

            let b_relations = match b {
                Some(v) => v,
                None => {
                    //無条件挿入
                    continue;
                }
            };

            //まずTopについて考える
        }
    }

    fn collect_other_dimension(
        dim: &BitVec,
        //ここを見て逆引きの参照先を見る
        dim_select: MainDimensionSelect,
        top_reverse: &Vec<&ReverseInfo>,
        under_reverse: &Vec<&ReverseInfo>,
    ) -> Option<(Vec<Relation>, Vec<Relation>)> {
        let mut top_disjoint = true;
        let mut under_disjoint = true;

        let mut top_relation: Vec<Relation> = Vec::new();
        let mut under_relation: Vec<Relation> = Vec::new();

        //代表次元における上位範囲を調べる

        for top in top_reverse {
            let target = match dim_select {
                MainDimensionSelect::F => &top.f,
                MainDimensionSelect::X => &top.x,
                MainDimensionSelect::Y => &top.y,
            };

            let relation = Self::check_relation(dim, target);

            if relation != Relation::Disjoint {
                top_disjoint = false;
            }

            top_relation.push(relation);
        }

        for under in under_reverse {
            let target = match dim_select {
                MainDimensionSelect::F => &under.f,
                MainDimensionSelect::X => &under.x,
                MainDimensionSelect::Y => &under.y,
            };

            let relation = Self::check_relation(dim, target);

            if relation != Relation::Disjoint {
                under_disjoint = false;
            }

            under_relation.push(relation);
        }

        if top_disjoint && under_disjoint {
            return None;
        } else {
            return Some((top_relation, under_relation));
        }
    }
}

//各軸について処理させる
