use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{
        Index, SpaceTimeIdSet,
        insert::{check_relation::Relation, insert_main_dim::MainDimensionSelect},
    },
};

pub enum ResultTop {
    ///上位のIDに含まれるため、これ以上やる必要がない
    End,

    ///上位のIDに含まれなかった、下位のIDも調べる必要がある
    Continue,
}
impl SpaceTimeIdSet {
    pub fn scan_and_insert_top(
        &mut self,
        main_bit: &BitVec,
        main_top: &HashSet<Index>,
        other_encoded: &[&Vec<(usize, BitVec)>; 2],
        main_dim_select: MainDimensionSelect,
        main_under_count: &usize,
    ) -> ResultTop {
        // コピーは破壊的操作用
        let mut other_encoded_copy: [Vec<Option<(usize, BitVec)>>; 2] =
            other_encoded.map(|v| v.iter().cloned().map(Some).collect());

        // 軸を動的に決定
        let main_idx = main_dim_select.as_index();
        let other_axes: [usize; 2] = match main_idx {
            0 => [1, 2], // F -> X,Y
            1 => [0, 2], // X -> F,Y
            2 => [0, 1], // Y -> F,X
            _ => unreachable!(),
        };

        for index in main_top {
            let reverse = self.reverse.get(index).unwrap();
            let target_bits = [&reverse.f.clone(), &reverse.x.clone(), &reverse.y.clone()];
            let mut target_main = target_bits[main_idx].clone();

            // 他軸の参照を動的に取得
            let target_a = &target_bits[other_axes[0]];
            let target_b = &target_bits[other_axes[1]];

            // 2軸間で共通ロジックを使う
            let mut a_relations = Vec::new();
            let mut b_relations = Vec::new();

            // ---- A軸を処理 ----
            for (i, (_, bit_a)) in other_encoded[0].iter().enumerate() {
                if let Some(_a_v) = other_encoded_copy[0][i].as_mut() {
                    let relation = Self::check_relation(bit_a, target_a);

                    if relation == Relation::Disjoint {
                        // 論理削除しつつ a_v を取り出す
                        let removed = other_encoded_copy[0][i].take().unwrap();

                        for b_opt in &other_encoded_copy[1] {
                            if let Some(b_v) = b_opt.as_ref() {
                                self.uncheck_insert(main_bit, &removed.1, &b_v.1);
                            }
                        }
                    } else {
                        a_relations.push((i, relation));
                    }
                }
            }

            // ---- B軸を処理 ----
            for (i, (_, bit_b)) in other_encoded[1].iter().enumerate() {
                if let Some(_b_v) = other_encoded_copy[1][i].as_mut() {
                    let relation = Self::check_relation(bit_b, target_b);

                    if relation == Relation::Disjoint {
                        let removed = other_encoded_copy[1][i].take().unwrap();

                        for a_opt in &other_encoded_copy[0] {
                            if let Some(a_v) = a_opt.as_ref() {
                                self.uncheck_insert(main_bit, &a_v.1, &removed.1);
                            }
                        }
                    } else {
                        b_relations.push((i, relation));
                    }
                }
            }

            // ---- メイン軸を含めた結合処理 ----
            for (ai, a_rel) in &a_relations {
                for (bi, b_rel) in &b_relations {
                    let a_opt = &other_encoded_copy[0][*ai];
                    let b_opt = &other_encoded_copy[1][*bi];

                    // どちらかが None ならスキップ
                    let a_v = match a_opt.as_ref() {
                        Some(val) => val,
                        None => continue,
                    };
                    let b_v = match b_opt.as_ref() {
                        Some(val) => val,
                        None => continue,
                    };

                    // 全てが上位の場合
                    if (*a_rel == Relation::Top) && (*b_rel == Relation::Top) {
                        return ResultTop::End;
                    }

                    // A軸が上位の場合
                    if *a_rel == Relation::Top {
                        let mut b_clone = b_v.1.clone();
                        let split_b = Self::split_dimension(target_b, &mut b_clone);
                        for bit_b in split_b {
                            self.uncheck_insert(main_bit, &a_v.1, &bit_b);
                        }
                    }

                    // B軸が上位の場合
                    if *b_rel == Relation::Top {
                        let mut a_clone = a_v.1.clone();
                        let split_a = Self::split_dimension(target_a, &mut a_clone);
                        for bit_a in split_a {
                            self.uncheck_insert(main_bit, &bit_a, &b_v.1);
                        }
                    }

                    // main軸を分割挿入
                    for bit_main in Self::split_dimension(main_bit, &mut target_main) {
                        self.insert_main_dim(
                            &bit_main,
                            &0,
                            main_under_count,
                            &mut vec![(main_under_count.clone(), main_bit.clone())],
                            other_encoded,
                            main_dim_select.clone(),
                        );
                    }
                }
            }
        }
        return ResultTop::Continue;
    }
}
