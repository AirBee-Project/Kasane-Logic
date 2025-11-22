use std::collections::HashSet;

use itertools::iproduct;

use crate::{
    bit_vec::{BitVec, relation::BitVecRelation},
    space_time_id_set::{
        Index, Interval, ReverseInfo, SpaceTimeIdSet, insert::under_under_top::NeedDivison,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum DimensionSelect {
    F,
    X,
    Y,
}

impl DimensionSelect {
    pub fn as_index(&self) -> usize {
        match self {
            DimensionSelect::F => 0,
            DimensionSelect::X => 1,
            DimensionSelect::Y => 2,
        }
    }
}

impl SpaceTimeIdSet {
    /// 代表次元×他の次元を挿入処理する
    pub(crate) fn insert_main_dim(
        &mut self,
        main_bit: &BitVec,
        main_index: &Index,
        main_under_count: &usize,
        main_encoded: &mut Vec<(Index, BitVec)>,
        other_encoded: &[&Vec<(Index, BitVec)>; 2],
        main_dim_select: DimensionSelect,
    ) {
        let main_under: Vec<Index> = Self::collect_top(&self, main_bit, &main_dim_select);

        if main_under.is_empty() && *main_under_count == 0 {
            for ((_, a_bit), (_, b_bit)) in iproduct!(other_encoded[0], other_encoded[1]) {
                match main_dim_select {
                    DimensionSelect::F => self.uncheck_insert(main_bit, a_bit, b_bit),
                    DimensionSelect::X => self.uncheck_insert(a_bit, main_bit, b_bit),
                    DimensionSelect::Y => self.uncheck_insert(a_bit, b_bit, main_bit),
                };
            }
            let _removed = main_encoded.remove(*main_index);
            return;
        }

        let main_top: Vec<Index> = self.collect_under(main_bit, &main_dim_select);

        let mut top_reverse = vec![];
        for top_index in &main_top {
            top_reverse.push(
                self.reverse
                    .get(&top_index)
                    .expect("Internal error: reverse index not found for top"),
            );
        }

        let mut under_reverse = vec![];
        for under_index in &main_under {
            under_reverse.push(
                self.reverse
                    .get(&*under_index)
                    .expect("Internal error: reverse index not found for under"),
            );
        }

        let a_dim_select: DimensionSelect;
        let b_dim_select: DimensionSelect;

        match main_dim_select {
            DimensionSelect::F => {
                a_dim_select = DimensionSelect::X;
                b_dim_select = DimensionSelect::Y;
            }
            DimensionSelect::X => {
                a_dim_select = DimensionSelect::F;
                b_dim_select = DimensionSelect::Y;
            }
            DimensionSelect::Y => {
                a_dim_select = DimensionSelect::F;
                b_dim_select = DimensionSelect::X;
            }
        }

        let mut a_relations: Vec<Option<(Vec<BitVecRelation>, Vec<BitVecRelation>)>> = Vec::new();
        let mut b_relations: Vec<Option<(Vec<BitVecRelation>, Vec<BitVecRelation>)>> = Vec::new();

        for (_, a_dim) in other_encoded[0] {
            a_relations.push(Self::collect_other_dimension(
                a_dim,
                a_dim_select,
                &top_reverse,
                &under_reverse,
            ));
        }

        for (_, b_dim) in other_encoded[1] {
            b_relations.push(Self::collect_other_dimension(
                b_dim,
                b_dim_select,
                &top_reverse,
                &under_reverse,
            ));
        }

        let mut need_delete: HashSet<Index> = HashSet::new();
        let mut need_insert: HashSet<ReverseInfo> = HashSet::new();

        'outer: for ((a_encode_index, a), (b_encode_index, b)) in iproduct!(
            a_relations.iter().enumerate(),
            b_relations.iter().enumerate()
        ) {
            let a_relation = match a {
                Some(v) => v,
                None => {
                    self.uncheck_insert_dim(
                        main_dim_select,
                        main_bit,
                        &other_encoded[0][a_encode_index].1,
                        &other_encoded[1][b_encode_index].1,
                    );
                    continue;
                }
            };

            let b_relation = match b {
                Some(v) => v,
                None => {
                    self.uncheck_insert_dim(
                        main_dim_select,
                        main_bit,
                        &other_encoded[0][a_encode_index].1,
                        &other_encoded[1][b_encode_index].1,
                    );
                    continue;
                }
            };

            let mut need_divison = NeedDivison {
                f: vec![],
                x: vec![],
                y: vec![],
            };

            let mut need_delete_inside: HashSet<Index> = HashSet::new();
            let mut need_insert_inside: HashSet<ReverseInfo> = HashSet::new();

            for (i, (a_rel, b_rel)) in a_relation.0.iter().zip(b_relation.0.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (
                        BitVecRelation::Greater | BitVecRelation::Equal,
                        BitVecRelation::Greater | BitVecRelation::Equal,
                    ) => {
                        need_delete_inside.insert(main_top[i]);
                    }
                    (BitVecRelation::Greater | BitVecRelation::Equal, BitVecRelation::Less) => {
                        self.top_top_under(
                            main_top[i],
                            other_encoded[1][b_encode_index].1.clone(),
                            b_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (BitVecRelation::Less, BitVecRelation::Greater | BitVecRelation::Equal) => {
                        self.top_top_under(
                            main_top[i],
                            other_encoded[0][a_encode_index].1.clone(),
                            a_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (BitVecRelation::Less, BitVecRelation::Less) => {
                        self.under_under_top(&mut need_divison, main_top[i], main_dim_select);
                    }
                    _ => {}
                }
            }

            for (i, (a_rel, b_rel)) in a_relation.1.iter().zip(b_relation.1.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (
                        BitVecRelation::Greater | BitVecRelation::Equal,
                        BitVecRelation::Greater | BitVecRelation::Equal,
                    ) => {
                        self.top_top_under(
                            main_under[i],
                            main_bit.clone(),
                            main_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (BitVecRelation::Greater | BitVecRelation::Equal, BitVecRelation::Less) => {
                        self.under_under_top(&mut need_divison, main_under[i], a_dim_select);
                    }
                    (BitVecRelation::Less, BitVecRelation::Greater | BitVecRelation::Equal) => {
                        self.under_under_top(&mut need_divison, main_under[i], b_dim_select);
                    }
                    (BitVecRelation::Less, BitVecRelation::Less) => {
                        continue 'outer;
                    }
                    _ => {}
                }
            }

            let f_splited;
            let x_splited;
            let y_splited;

            match main_dim_select {
                DimensionSelect::F => {
                    f_splited = BitVec::division(main_bit.clone(), need_divison.f);

                    x_splited = BitVec::division(
                        other_encoded[0][a_encode_index].1.clone(),
                        need_divison.x,
                    );

                    y_splited = BitVec::division(
                        other_encoded[1][b_encode_index].1.clone(),
                        need_divison.y,
                    );
                }
                DimensionSelect::X => {
                    f_splited = BitVec::division(
                        other_encoded[0][a_encode_index].1.clone(),
                        need_divison.f,
                    );

                    x_splited = BitVec::division(main_bit.clone(), need_divison.x);

                    y_splited = BitVec::division(
                        other_encoded[1][b_encode_index].1.clone(),
                        need_divison.y,
                    );
                }
                DimensionSelect::Y => {
                    f_splited = BitVec::division(
                        other_encoded[0][a_encode_index].1.clone(),
                        need_divison.f,
                    );

                    x_splited = BitVec::division(
                        other_encoded[1][b_encode_index].1.clone(),
                        need_divison.x,
                    );

                    y_splited = BitVec::division(main_bit.clone(), need_divison.y);
                }
            }

            for (f, x, y) in iproduct!(f_splited, x_splited, y_splited) {
                self.uncheck_insert(&f, &x, &y);
            }

            need_delete.extend(need_delete_inside);
            need_insert.extend(need_insert_inside);
        }
        for need_delete_index in need_delete {
            self.uncheck_delete(&need_delete_index);
        }
        for reverse in need_insert {
            self.uncheck_insert(&reverse.f, &reverse.x, &reverse.y);
        }

        main_encoded.remove(*main_index);
    }
}
