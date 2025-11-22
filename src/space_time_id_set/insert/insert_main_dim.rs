use std::collections::{HashSet, btree_map::Range};

use itertools::iproduct;

use crate::{
    bit_vec::{HierarchicalKey, relation::HierarchicalKeyRelation},
    space_time_id_set::{
        Index, Interval, ReverseInfo, SpaceTimeIdSet,
        insert::{select_dimensions::DimensionSelect, split_self::RangesCollect},
    },
};

impl SpaceTimeIdSet {
    /// 代表次元×他の次元を挿入処理する
    pub(crate) fn insert_main_dim(
        &mut self,
        main_bit: &HierarchicalKey,
        main_index: &Index,
        main_descendant_count: &usize,
        main_encoded: &mut Vec<(Index, HierarchicalKey)>,
        a_encoded: &Vec<(Index, HierarchicalKey)>,
        b_encoded: &Vec<(Index, HierarchicalKey)>,
        main_dim_select: DimensionSelect,
    ) {
        let main_ancestors: Vec<Index> = Self::collect_ancestors(&self, main_bit, &main_dim_select);

        //Main次元において、祖先にも子孫にも重なる範囲が存在しなければ挿入
        if main_ancestors.is_empty() && *main_descendant_count == 0 {
            for ((_, a_bit), (_, b_bit)) in iproduct!(a_encoded, b_encoded) {
                self.uncheck_insert(main_bit, a_bit, b_bit, &main_dim_select);
            }
            let _removed = main_encoded.remove(*main_index);
            return;
        }

        //Main次元における子孫のIndexを取得する
        let main_descendants: Vec<Index> = self.collect_descendants(main_bit, &main_dim_select);

        //Main次元における子孫を逆引き情報で取得する
        let mut main_descendants_reverse = vec![];
        for top_index in &main_descendants {
            main_descendants_reverse.push(
                self.reverse
                    .get(&top_index)
                    .expect("Internal error: reverse index not found for top"),
            );
        }

        //Main次元における祖先のIndexを取得する
        let mut main_ancestors_reverse = vec![];

        //Main次元における祖先を逆引き情報で取得する
        for under_index in &main_ancestors {
            main_ancestors_reverse.push(
                self.reverse
                    .get(&*under_index)
                    .expect("Internal error: reverse index not found for under"),
            );
        }

        let mut a_relations: Vec<Option<(Vec<HierarchicalKeyRelation>, Vec<HierarchicalKeyRelation>)>> = Vec::new();
        let mut b_relations: Vec<Option<(Vec<HierarchicalKeyRelation>, Vec<HierarchicalKeyRelation>)>> = Vec::new();

        for (_, a_dim) in a_encoded {
            a_relations.push(Self::collect_other_dimension(
                a_dim,
                &main_dim_select.a(),
                &main_descendants_reverse,
                &main_ancestors_reverse,
            ));
        }

        for (_, b_dim) in b_encoded {
            b_relations.push(Self::collect_other_dimension(
                b_dim,
                &main_dim_select.b(),
                &main_descendants_reverse,
                &main_ancestors_reverse,
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
                    self.uncheck_insert(
                        main_bit,
                        &a_encoded[a_encode_index].1,
                        &b_encoded[b_encode_index].1,
                        &main_dim_select,
                    );
                    continue;
                }
            };

            let b_relation = match b {
                Some(v) => v,
                None => {
                    self.uncheck_insert(
                        main_bit,
                        &a_encoded[a_encode_index].1,
                        &b_encoded[b_encode_index].1,
                        &main_dim_select,
                    );
                    continue;
                }
            };

            let mut need_divison = RangesCollect {
                main: vec![],
                a: vec![],
                b: vec![],
            };

            let mut need_delete_inside: HashSet<Index> = HashSet::new();
            let mut need_insert_inside: HashSet<ReverseInfo> = HashSet::new();

            for (i, (a_rel, b_rel)) in a_relation.0.iter().zip(b_relation.0.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                    ) => {
                        need_delete_inside.insert(main_descendants[i]);
                    }
                    (
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                        HierarchicalKeyRelation::Descendant,
                    ) => {
                        self.split_other(
                            main_descendants[i],
                            b_encoded[b_encode_index].1.clone(),
                            &main_dim_select.b(),
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (
                        HierarchicalKeyRelation::Descendant,
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                    ) => {
                        self.split_other(
                            main_descendants[i],
                            a_encoded[a_encode_index].1.clone(),
                            &main_dim_select.a(),
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (HierarchicalKeyRelation::Descendant, HierarchicalKeyRelation::Descendant) => {
                        self.split_self(&mut need_divison, main_descendants[i], &main_dim_select);
                    }
                    _ => {}
                }
            }

            for (i, (a_rel, b_rel)) in a_relation.1.iter().zip(b_relation.1.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                    ) => {
                        self.split_other(
                            main_ancestors[i],
                            main_bit.clone(),
                            &main_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                        HierarchicalKeyRelation::Descendant,
                    ) => {
                        self.split_self(&mut need_divison, main_ancestors[i], &main_dim_select.a());
                    }
                    (
                        HierarchicalKeyRelation::Descendant,
                        HierarchicalKeyRelation::Ancestor | HierarchicalKeyRelation::Equal,
                    ) => {
                        self.split_self(&mut need_divison, main_ancestors[i], &main_dim_select.b());
                    }
                    (HierarchicalKeyRelation::Descendant, HierarchicalKeyRelation::Descendant) => {
                        continue 'outer;
                    }
                    _ => {}
                }
            }

            let main_splited = main_bit.subtract_ranges(&need_divison.main);

            let a_splited = a_encoded[a_encode_index]
                .1
                .subtract_ranges(&need_divison.main);
            let b_splited = b_encoded[b_encode_index].1.subtract_ranges(&need_divison.b);

            for (main, a, b) in iproduct!(main_splited, a_splited, b_splited) {
                self.uncheck_insert(&main, &a, &b, &main_dim_select);
            }

            need_delete.extend(need_delete_inside);
            need_insert.extend(need_insert_inside);
        }
        for need_delete_index in need_delete {
            self.uncheck_delete(&need_delete_index);
        }
        for reverse in need_insert {
            self.uncheck_insert(&reverse.f, &reverse.x, &reverse.y, &DimensionSelect::F);
        }

        main_encoded.remove(*main_index);
    }
}
