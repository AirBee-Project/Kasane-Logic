use std::collections::HashSet;

use itertools::iproduct;

use crate::{
    bit_vec::relation::BitVecRelation,
    encode_id::EncodeID,
    space_time_id_set::{
        EncodeIDSet, Index,
        insert::{self, select_dimensions::DimensionSelect, split_self::RangesCollect},
    },
};
pub mod collect_ancestors;
pub mod collect_descendants;
pub mod collect_other_dimension;
pub mod generate_index;
pub mod select_dimensions;
pub mod split_other;
pub mod split_self;
pub mod uncheck_delete;
pub mod uncheck_insert;

impl EncodeIDSet {
    ///EncodeIDSetに新規のEncodeIDを挿入する。
    /// 既存の範囲と重複がある場合は挿入時に調整が行われ、重複が排除される。
    pub fn insert(&mut self, encode_id: EncodeID) {
        print!("Call Insert  ");
        println!("{:?}", encode_id);

        //下位IDの個数が最小の次元を選択する
        let f_descendants_count = match self.f.get(&encode_id.f) {
            Some(info) => info.count,
            None => 0,
        };

        let x_descendants_count = match self.x.get(&encode_id.f) {
            Some(info) => info.count,
            None => 0,
        };

        let y_descendants_count = match self.y.get(&encode_id.f) {
            Some(info) => info.count,
            None => 0,
        };

        let min_count = f_descendants_count.min(x_descendants_count.min(y_descendants_count));

        //代表の次元を選出する
        let main_dim;
        let a_dim;
        let b_dim;
        let main;
        let a;
        let b;

        if min_count == f_descendants_count {
            main_dim = DimensionSelect::F;
            a_dim = DimensionSelect::X;
            b_dim = DimensionSelect::Y;
            main = &encode_id.f;
            a = &encode_id.x;
            b = &encode_id.y;
        } else if min_count == x_descendants_count {
            main_dim = DimensionSelect::X;
            a_dim = DimensionSelect::F;
            b_dim = DimensionSelect::Y;
            main = &encode_id.x;
            a = &encode_id.f;
            b = &encode_id.y;
        } else {
            main_dim = DimensionSelect::Y;
            a_dim = DimensionSelect::F;
            b_dim = DimensionSelect::X;
            main = &encode_id.y;
            a = &encode_id.f;
            b = &encode_id.x;
        }

        //Main次元の祖先を探索する
        let main_ancestors: Vec<Index> = Self::collect_ancestors(&self, main, &main_dim);

        //Main次元において、祖先にも子孫にも重なる範囲が存在しなければ挿入
        if main_ancestors.is_empty() && min_count == 0 {
            self.uncheck_insert(encode_id);
            return;
        }

        //Main次元における祖先のIndexを取得する
        let mut main_ancestors_reverse = vec![];

        //Main次元における祖先を逆引き情報で取得する
        for ancestor_index in &main_ancestors {
            main_ancestors_reverse.push(
                self.reverse
                    .get(&*ancestor_index)
                    .expect("Internal error: reverse index not found for under"),
            );
        }

        //Main次元における子孫のIndexを取得する
        let main_descendants: Vec<Index> = self.collect_descendants(main, &main_dim);

        //Main次元における子孫を逆引き情報で取得する
        let mut main_descendants_reverse = vec![];
        for descendant_index in &main_descendants {
            main_descendants_reverse.push(
                self.reverse
                    .get(&descendant_index)
                    .expect("Internal error: reverse index not found for top"),
            );
        }

        let a_relation = match Self::collect_other_dimension(
            a,
            &main_ancestors_reverse,
            &main_descendants_reverse,
            &a_dim,
        ) {
            Some(v) => v,
            None => {
                self.uncheck_insert(encode_id);
                return;
            }
        };

        let b_relation = match Self::collect_other_dimension(
            b,
            &main_ancestors_reverse,
            &main_descendants_reverse,
            &b_dim,
        ) {
            Some(v) => v,
            None => {
                self.uncheck_insert(encode_id);
                return;
            }
        };

        let mut need_delete: HashSet<Index> = HashSet::new();
        let mut need_insert: HashSet<EncodeID> = HashSet::new();

        let mut collect_divison_ranges = RangesCollect {
            f: vec![],
            x: vec![],
            y: vec![],
        };

        //Main次元における祖先の範囲について調べる
        for (i, (a_rel, b_rel)) in a_relation.0.iter().zip(b_relation.0.iter()).enumerate() {
            match (a_rel, b_rel) {
                (
                    BitVecRelation::Ancestor | BitVecRelation::Equal,
                    BitVecRelation::Ancestor | BitVecRelation::Equal,
                ) => {
                    //全ての次元において子孫のIDが存在するため削除
                    need_delete.insert(main_descendants[i]);
                }
                (BitVecRelation::Ancestor | BitVecRelation::Equal, BitVecRelation::Descendant) => {
                    self.split_other(
                        main_ancestors[i],
                        b,
                        &main_dim.b(),
                        &mut need_delete,
                        &mut need_insert,
                    );
                }
                (BitVecRelation::Descendant, BitVecRelation::Ancestor | BitVecRelation::Equal) => {
                    self.split_other(
                        main_ancestors[i],
                        a,
                        &main_dim.a(),
                        &mut need_delete,
                        &mut need_insert,
                    );
                }
                (BitVecRelation::Descendant, BitVecRelation::Descendant) => {
                    self.split_self(&mut collect_divison_ranges, main_ancestors[i], &main_dim);
                }
                _ => {}
            }
        }

        for (i, (a_rel, b_rel)) in a_relation.1.iter().zip(b_relation.1.iter()).enumerate() {
            match (a_rel, b_rel) {
                (
                    BitVecRelation::Ancestor | BitVecRelation::Equal,
                    BitVecRelation::Ancestor | BitVecRelation::Equal,
                ) => {
                    self.split_other(
                        main_descendants[i],
                        main,
                        &main_dim,
                        &mut need_delete,
                        &mut need_insert,
                    );
                }
                (BitVecRelation::Ancestor | BitVecRelation::Equal, BitVecRelation::Descendant) => {
                    self.split_self(
                        &mut collect_divison_ranges,
                        main_descendants[i],
                        &main_dim.a(),
                    );
                }
                (BitVecRelation::Descendant, BitVecRelation::Ancestor | BitVecRelation::Equal) => {
                    self.split_self(
                        &mut collect_divison_ranges,
                        main_descendants[i],
                        &main_dim.b(),
                    );
                }
                (BitVecRelation::Descendant, BitVecRelation::Descendant) => {
                    //全ての次元において祖先のIDが存在するため、何もする必要がない
                    return;
                }
                _ => {}
            }
        }

        //既存IDのうち、削除すべきものを削除する
        for need_delete_index in need_delete {
            self.uncheck_delete(&need_delete_index);
        }

        //既存IDを分割した場合に、分割後の範囲を挿入する
        for id in need_insert {
            self.uncheck_insert(id);
        }

        //Main次元において、挿入するIDに切断が必要な部分を切断する
        let f_splited = encode_id.f.subtract_ranges(&collect_divison_ranges.f);

        //A次元において、挿入するIDに切断が必要な部分を切断する
        let x_splited = encode_id.x.subtract_ranges(&collect_divison_ranges.x);

        //B次元において、挿入するIDに切断が必要な部分を切断する
        let y_splited = encode_id.y.subtract_ranges(&collect_divison_ranges.y);

        //切断された範囲を挿入する
        for (f, x, y) in iproduct!(f_splited, x_splited, y_splited) {
            self.uncheck_insert(EncodeID { f, x, y });
        }
    }
}
