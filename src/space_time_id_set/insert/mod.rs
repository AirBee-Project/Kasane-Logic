use crate::{
    bit_vec::BitVec,
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        SpaceTimeIdSet,
        insert::insert_main_dim::MainDimensionSelect,
        single::{
            convert_bitvec_f::convert_bitmask_f, convert_bitvec_xy::convert_bitmask_xy,
            convert_single_f::convert_f, convert_single_xy::convert_xy,
        },
    },
};
pub mod check_relation;
pub mod collect_top;
pub mod collect_under;
pub mod generate_index;
pub mod insert_main_dim;
pub mod scan_and_insert_top;
pub mod scan_and_insert_under;
pub mod search_under_count;
pub mod select_dimensions;
pub mod split_dimension;
pub mod uncheck_delete;
pub mod uncheck_insert;
pub mod uncheck_insert_combinations;

impl SpaceTimeIdSet {
    pub fn insert(&mut self, id: SpaceTimeId) {
        //IDを各次元ごとに最適な単体範囲に分解する
        let f_splited = convert_f(id.z, id.f);
        let x_splited = convert_xy(id.z, id.x);
        let y_splited = convert_xy(id.z, id.y);

        //各次元の範囲をBitVecに変換する(usize=下位範囲の個数、BitVec=エンコード結果)
        let mut f_encoded: Vec<(usize, BitVec)> = f_splited
            .iter()
            .map(|(z, f)| {
                let bit_vec = convert_bitmask_f(*z, *f);
                (Self::search_under_count(&self.f, &bit_vec), bit_vec)
            })
            .collect();
        let mut x_encoded: Vec<(usize, BitVec)> = x_splited
            .iter()
            .map(|(z, x)| {
                let bit_vec = convert_bitmask_xy(*z, *x);
                (Self::search_under_count(&self.x, &bit_vec), bit_vec)
            })
            .collect();
        let mut y_encoded: Vec<(usize, BitVec)> = y_splited
            .iter()
            .map(|(z, y)| {
                let bit_vec = convert_bitmask_xy(*z, *y);
                (Self::search_under_count(&self.y, &bit_vec), bit_vec)
            })
            .collect();
        //最も探索範囲が小さくなりそうな次元を代表次元として挿入を繰り返す
        //どこかの次元がなくなるまで繰り返す

        while !(f_encoded.is_empty() || x_encoded.is_empty() || y_encoded.is_empty()) {
            //各次元の代表の最小のやつを求める
            let (f_index, f_under_min_val) = {
                let (i, v) = f_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .unwrap();
                (i, (v.0, v.1.clone())) // cloneしておく
            };

            let (x_index, x_under_min_val) = {
                let (i, v) = x_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .unwrap();
                (i, (v.0, v.1.clone()))
            };

            let (y_index, y_under_min_val) = {
                let (i, v) = y_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .unwrap();
                (i, (v.0, v.1.clone()))
            };

            let min_under = f_under_min_val
                .0
                .min(x_under_min_val.0.min(y_under_min_val.0));

            if min_under == f_under_min_val.0 {
                self.insert_main_dim(
                    &f_under_min_val.1,
                    &f_index,
                    &min_under,
                    &mut f_encoded,
                    &[&x_encoded, &y_encoded],
                    MainDimensionSelect::F,
                );
            } else if min_under == x_under_min_val.0 {
                self.insert_main_dim(
                    &x_under_min_val.1,
                    &x_index,
                    &min_under,
                    &mut x_encoded,
                    &[&f_encoded, &y_encoded],
                    MainDimensionSelect::F,
                );
            } else {
                self.insert_main_dim(
                    &y_under_min_val.1,
                    &y_index,
                    &min_under,
                    &mut y_encoded,
                    &[&f_encoded, &x_encoded],
                    MainDimensionSelect::F,
                );
            }
        }
    }
}
