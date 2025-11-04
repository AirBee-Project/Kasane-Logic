use serde::de::value;

use crate::{
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        Index, SpaceTimeIdSet,
        single::{
            convert_bitvec_f::convert_bitmask_f,
            convert_bitvec_xy::convert_bitmask_xy,
            convert_single_f::{self, convert_f},
            convert_single_xy::convert_xy,
        },
    },
    r#type::bit_vec::BitVec,
};
pub mod search_under;
pub mod tmp;

impl SpaceTimeIdSet {
    pub fn insert(&self, id: SpaceTimeId) {
        //IDを各次元ごとに最適な単体範囲に分解する
        let f_splited = convert_f(id.z, id.f);
        let x_splited = convert_xy(id.z, id.x);
        let y_splited = convert_xy(id.z, id.y);

        //各次元の範囲をBitVecに変換する
        let mut f_encoded: Vec<(usize, BitVec)> = f_splited
            .iter()
            .map(|(z, f)| {
                let bit_vec = convert_bitmask_f(*z, *f);
                (Self::search_under(&self.f, &bit_vec), bit_vec)
            })
            .collect();
        let mut x_encoded: Vec<(usize, BitVec)> = x_splited
            .iter()
            .map(|(z, x)| {
                let bit_vec = convert_bitmask_xy(*z, *x);
                (Self::search_under(&self.x, &bit_vec), bit_vec)
            })
            .collect();
        let mut y_encoded: Vec<(usize, BitVec)> = y_splited
            .iter()
            .map(|(z, y)| {
                let bit_vec = convert_bitmask_xy(*z, *y);
                (Self::search_under(&self.y, &bit_vec), bit_vec)
            })
            .collect();

        while !(f_encoded.is_empty() || x_encoded.is_empty() || y_encoded.is_empty()) {
            //各次元の代表の最小のやつを求める
            let f_under_min = f_encoded
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.0)
                .unwrap();
            let x_under_min = x_encoded
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.0)
                .unwrap();
            let y_under_min = y_encoded
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.0)
                .unwrap();

            //全ての次元において最小のものを求める
            let min_under = f_under_min.1.0.min(x_under_min.1.0.min(y_under_min.1.0));

            //代表次元を選定して、関数を実行する
            if min_under == f_under_min.1.0 {
                //Fが代表次元
                Self::tmp(&self.f, &[&self.x, &self.y], &min_under, &f_under_min.1.1);
            } else if min_under == x_under_min.1.0 {
                //Xが代表次元
                Self::tmp(&self.x, &[&self.f, &self.y], &min_under, &x_under_min.1.1);
            } else {
                //Yが代表次元
                Self::tmp(&self.y, &[&self.f, &self.x], &min_under, &y_under_min.1.1);
            }
        }

        //基準にする軸を決めるために、全ての軸で探索を行う

        //分離範囲ごとに下位IDの個数を調べる

        //下位IDの個数が少ない順にSortする

        //挿入していく

        //上位IDを調べる

        //上位IDがある場合は逆引きして他の次元と重なりがないかを検証する

        //この段階で代表次元について上位IDと下位IDが出そろう
        //順番に逆引きしていく
        //上位IDの場合は挿入しない
        //下位IDの場合は下位IDを削除
        //部分の場合は総合して下位を切る
        //隣に連続なIDがあればくっつける

        //これを繰り返す
    }
}
