use std::collections::HashSet;

use itertools::iproduct;

use crate::{
    bit_vec::BitVec,
    space_time_id_set::{
        Index, ReverseInfo, SpaceTimeIdSet,
        insert::{check_relation::Relation, under_under_top::NeedDivison},
    },
};

/// 次元の選択を表す列挙型
///
/// 時空間IDの3次元（F: 高度、X: 経度、Y: 緯度）のいずれかを表す。
#[derive(Clone, Copy, Debug)]
pub enum DimensionSelect {
    /// F次元（高度）
    F,
    /// X次元（経度）
    X,
    /// Y次元（緯度）
    Y,
}

impl DimensionSelect {
    /// 次元をインデックス番号に変換する
    ///
    /// # 戻り値
    /// F=0, X=1, Y=2
    pub fn as_index(&self) -> usize {
        match self {
            DimensionSelect::F => 0,
            DimensionSelect::X => 1,
            DimensionSelect::Y => 2,
        }
    }
}

impl SpaceTimeIdSet {
    /// 代表次元を基準に他の次元との組み合わせを挿入処理する
    ///
    /// 指定された代表次元と他の2次元を組み合わせて、既存のIDとの包含関係を解決しながら挿入する。
    /// 最も効率的な次元を代表として選択し、処理を行う。
    ///
    /// # 引数
    /// * `main_bit` - 代表次元のBitVec
    /// * `main_index` - main_encoded内でのインデックス
    /// * `main_under_count` - 代表次元の下位IDの数
    /// * `main_encoded` - 代表次元のエンコード済みリスト
    /// * `other_encoded` - 他の2次元のエンコード済みリスト
    /// * `main_dim_select` - 代表次元の選択
    pub fn insert_main_dim(
        &mut self,
        main_bit: &BitVec,
        main_index: &Index,
        main_under_count: &usize,
        main_encoded: &mut Vec<(Index, BitVec)>,
        other_encoded: &[&Vec<(Index, BitVec)>; 2],
        main_dim_select: DimensionSelect,
    ) {
        println!("代表次元：{:?}", main_dim_select);
        //代表次元における上位範囲を収拾する
        let main_top: Vec<Index> = Self::collect_top(self, main_bit, &main_dim_select);

        //代表次元において、上位も下位も存在しなかった場合は無条件に挿入
        if main_top.is_empty() && *main_under_count == 0 {
            //挿入
            for ((_, a_bit), (_, b_bit)) in iproduct!(other_encoded[0], other_encoded[1]) {
                match main_dim_select {
                    DimensionSelect::F => self.uncheck_insert(main_bit, a_bit, b_bit),
                    DimensionSelect::X => self.uncheck_insert(a_bit, main_bit, b_bit),
                    DimensionSelect::Y => self.uncheck_insert(a_bit, b_bit, main_bit),
                };

                //代表次元を元の要素から削除
            }
            let _removed = main_encoded.remove(*main_index);
            return;
        }

        //代表次元において下位の範囲を収拾
        let main_under: Vec<Index> = self.collect_under(main_bit, &main_dim_select);

        //逆引き
        let mut top_reverse = vec![];
        for top_index in &main_top {
            top_reverse.push(self.reverse.get(top_index).unwrap());
        }

        //逆引き
        let mut under_reverse = vec![];
        for under_index in &main_under {
            under_reverse.push(self.reverse.get(under_index).unwrap());
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

        //既にあるIDから削除するための構造体を作成
        let mut need_delete: HashSet<Index> = HashSet::new();
        let mut need_insert: HashSet<ReverseInfo> = HashSet::new();

        'outer: for ((a_encode_index, a), (b_encode_index, b)) in iproduct!(
            a_relations.iter().enumerate(),
            b_relations.iter().enumerate()
        ) {
            //もしA軸が無関係ならば即時挿入する
            let a_relation = match a {
                Some(v) => v,
                None => {
                    println!("無条件挿入");
                    self.uncheck_insert_dim(
                        main_dim_select,
                        main_bit,
                        &other_encoded[0][a_encode_index].1,
                        &other_encoded[1][b_encode_index].1,
                    );
                    continue;
                }
            };

            //もしB軸が無関係ならば即時挿入する
            let b_relation = match b {
                Some(v) => v,
                None => {
                    println!("無条件挿入");

                    self.uncheck_insert_dim(
                        main_dim_select,
                        main_bit,
                        &other_encoded[0][a_encode_index].1,
                        &other_encoded[1][b_encode_index].1,
                    );
                    continue;
                }
            };

            //自分から削除する部分をためる構造体を作成
            let mut need_divison = NeedDivison {
                f: vec![],
                x: vec![],
                y: vec![],
            };

            //ここに来るということはAもBも関係があるので順番に競合を解消してあげる

            //代表次元におけるTopから処理する
            println!("{:?}", a_relation);
            println!("{:?}", b_relation);

            //削除するべきものをまとめる
            let mut need_delete_inside: HashSet<Index> = HashSet::new();
            let mut need_insert_inside: HashSet<ReverseInfo> = HashSet::new();

            for (i, (a_rel, b_rel)) in a_relation.0.iter().zip(b_relation.0.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (Relation::Top, Relation::Top) => {
                        continue 'outer;
                    }
                    (Relation::Top, Relation::Under) => {
                        println!("TTU");

                        //相手を切断
                        self.top_top_under(
                            main_top[i],
                            other_encoded[1][b_encode_index].1.clone(),
                            b_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (Relation::Under, Relation::Top) => {
                        println!("TUT");

                        //相手を切断
                        self.top_top_under(
                            main_top[i],
                            other_encoded[0][a_encode_index].1.clone(),
                            a_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (Relation::Under, Relation::Under) => {
                        println!("TUU");

                        //自分を削る
                        self.under_under_top(&mut need_divison, main_top[i], main_dim_select);
                    }
                    _ => {}
                }
            }

            //代表次元におけるUnderを処理する
            for (i, (a_rel, b_rel)) in a_relation.1.iter().zip(b_relation.1.iter()).enumerate() {
                match (a_rel, b_rel) {
                    (Relation::Top, Relation::Top) => {
                        println!("UTT");
                        //相手を切断
                        self.top_top_under(
                            main_under[i],
                            main_bit.clone(),
                            main_dim_select,
                            &mut need_delete_inside,
                            &mut need_insert_inside,
                        );
                    }
                    (Relation::Top, Relation::Under) => {
                        println!("UTU");

                        //自分を切断
                        self.under_under_top(&mut need_divison, main_under[i], a_dim_select);
                    }
                    (Relation::Under, Relation::Top) => {
                        println!("UUT");

                        //自分を切断
                        self.under_under_top(&mut need_divison, main_under[i], b_dim_select);
                    }
                    (Relation::Under, Relation::Under) => {
                        println!("UUU");

                        //下位のIDを削除
                        self.uncheck_delete(&main_under[i]);
                    }
                    _ => {}
                }
            }

            println!("{:?}", need_divison);

            //自身を分割
            let f_splited;
            let x_splited;
            let y_splited;

            println!("{:?}", main_dim_select);
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

            println!("INSIDE_DELETE:{:?}", need_delete_inside);
            println!("INSIDE_INSERT:{:?}", need_insert_inside);

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
