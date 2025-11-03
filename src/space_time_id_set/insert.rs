use crate::{space_time_id::SpaceTimeId, space_time_id_set::{SpaceTimeIdSet, single::{convert_bitvec_f::convert_bitmask_f, convert_single_f::{self, convert_f}}}};

impl SpaceTimeIdSet {
    pub fn insert(&mut self, space_time_id: SpaceTimeId) {
        //IDを各次元ごとに最適な単体範囲に分解する
        let f_single=convert_f(z, dim)

        //各次元の範囲をBitVecに変換する

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
