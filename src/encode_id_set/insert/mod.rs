use std::collections::HashSet;

use itertools::iproduct;

use crate::{
    bit_vec::relation::BitVecRelation,
    encode_id::EncodeID,
    encode_id_set::{
        EncodeIDSet, Index,
        insert::{self, split_self::RangesCollect},
        utils::select_dimensions::DimensionSelect,
    },
};
pub mod split_other;
pub mod split_self;

impl EncodeIDSet {
    ///EncodeIDSetに新規のEncodeIDを挿入する。
    /// 既存の範囲と重複がある場合は挿入時に調整が行われ、重複が排除される。
    pub fn insert(&mut self, encode_id: EncodeID) {
        //下位IDの個数が最小の次元を選択する
        let f_descendants_count = match self.f.get(&encode_id.f) {
            Some(info) => info.count,
            None => 0,
        };

        let x_descendants_count = match self.x.get(&encode_id.x) {
            Some(info) => info.count,
            None => 0,
        };

        let y_descendants_count = match self.y.get(&encode_id.y) {
            Some(info) => info.count,
            None => 0,
        };

        let t_descendants_count = match self.t.get(&encode_id.t) {
            Some(info) => info.count,
            None => 0,
        };

        let min_count = f_descendants_count.min(x_descendants_count.min(y_descendants_count.min(t_descendants_count)));

        //代表の次元を選出する
        let main_dim;
        let a_dim;
        let b_dim;
        let c_dim;
        let main;
        let a;
        let b;
        let c;

        if min_count == f_descendants_count {
            main_dim = DimensionSelect::F;
            a_dim = DimensionSelect::X;
            b_dim = DimensionSelect::Y;
            c_dim = DimensionSelect::T;
            main = &encode_id.f;
            a = &encode_id.x;
            b = &encode_id.y;
            c = &encode_id.t;
        } else if min_count == x_descendants_count {
            main_dim = DimensionSelect::X;
            a_dim = DimensionSelect::F;
            b_dim = DimensionSelect::Y;
            c_dim = DimensionSelect::T;
            main = &encode_id.x;
            a = &encode_id.f;
            b = &encode_id.y;
            c = &encode_id.t;
        } else if min_count == y_descendants_count {
            main_dim = DimensionSelect::Y;
            a_dim = DimensionSelect::F;
            b_dim = DimensionSelect::X;
            c_dim = DimensionSelect::T;
            main = &encode_id.y;
            a = &encode_id.f;
            b = &encode_id.x;
            c = &encode_id.t;
        } else {
            main_dim = DimensionSelect::T;
            a_dim = DimensionSelect::F;
            b_dim = DimensionSelect::X;
            c_dim = DimensionSelect::Y;
            main = &encode_id.t;
            a = &encode_id.f;
            b = &encode_id.x;
            c = &encode_id.y;
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

        let c_relation = match Self::collect_other_dimension(
            c,
            &main_ancestors_reverse,
            &main_descendants_reverse,
            &c_dim,
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
            t: vec![],
        };

        //Main次元における祖先の範囲を調べる
        for (i, ((a_rel, b_rel), c_rel)) in a_relation.0.iter().zip(b_relation.0.iter()).zip(c_relation.0.iter()).enumerate() {
            let a_desc = matches!(a_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let b_desc = matches!(b_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let c_desc = matches!(c_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let a_anc = matches!(a_rel, BitVecRelation::Ancestor);
            let b_anc = matches!(b_rel, BitVecRelation::Ancestor);
            let c_anc = matches!(c_rel, BitVecRelation::Ancestor);

            if a_desc && b_desc && c_desc {
                // 全ての次元において子孫のIDが存在する
                self.split_other(
                    &main_ancestors[i],
                    main_ancestors_reverse[i],
                    main,
                    &main_dim,
                    &mut need_delete,
                    &mut need_insert,
                );
            } else if a_anc && b_anc && c_anc {
                // 全ての次元において祖先のIDが存在するため、何もする必要がない
                return;
            } else if a_anc && b_desc && c_desc {
                // a次元のみ祖先
                self.split_self(
                    main_ancestors_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim.a(),
                );
            } else if b_anc && a_desc && c_desc {
                // b次元のみ祖先
                self.split_self(
                    main_ancestors_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim.b(),
                );
            } else if c_anc && a_desc && b_desc {
                // c次元のみ祖先
                self.split_self(
                    main_ancestors_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim.c(),
                );
            }
        }

        //Main次元における子孫の範囲について調べる
        for (i, ((a_rel, b_rel), c_rel)) in a_relation.1.iter().zip(b_relation.1.iter()).zip(c_relation.1.iter()).enumerate() {
            let a_desc = matches!(a_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let b_desc = matches!(b_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let c_desc = matches!(c_rel, BitVecRelation::Descendant | BitVecRelation::Equal);
            let a_anc = matches!(a_rel, BitVecRelation::Ancestor);
            let b_anc = matches!(b_rel, BitVecRelation::Ancestor);
            let c_anc = matches!(c_rel, BitVecRelation::Ancestor);

            if a_desc && b_desc && c_desc {
                // 全ての次元において子孫のIDが存在するため削除
                need_delete.insert(main_descendants[i]);
            } else if a_desc && b_desc && c_anc {
                // c次元のみ祖先
                self.split_other(
                    &main_ancestors[i],
                    main_descendants_reverse[i],
                    c,
                    &main_dim.c(),
                    &mut need_delete,
                    &mut need_insert,
                );
            } else if a_desc && b_anc && c_desc {
                // b次元のみ祖先
                self.split_other(
                    &main_ancestors[i],
                    main_descendants_reverse[i],
                    b,
                    &main_dim.b(),
                    &mut need_delete,
                    &mut need_insert,
                );
            } else if a_anc && b_desc && c_desc {
                // a次元のみ祖先
                self.split_other(
                    &main_ancestors[i],
                    main_descendants_reverse[i],
                    a,
                    &main_dim.a(),
                    &mut need_delete,
                    &mut need_insert,
                );
            } else if a_anc && b_anc && c_desc {
                // a,b次元が祖先
                self.split_self(
                    main_descendants_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim,
                );
            } else if a_anc && b_desc && c_anc {
                // a,c次元が祖先
                self.split_self(
                    main_descendants_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim,
                );
            } else if a_desc && b_anc && c_anc {
                // b,c次元が祖先
                self.split_self(
                    main_descendants_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim,
                );
            } else if a_anc && b_anc && c_anc {
                // 全て祖先
                self.split_self(
                    main_descendants_reverse[i],
                    &mut collect_divison_ranges,
                    &main_dim,
                );
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

        //C次元において、挿入するIDに切断が必要な部分を切断する
        let t_splited = encode_id.t.subtract_ranges(&collect_divison_ranges.t);

        //切断された範囲を挿入する
        for (f, x, y, t) in iproduct!(f_splited, x_splited, y_splited, t_splited) {
            self.uncheck_insert(EncodeID { f, x, y, t });
        }
    }
}
