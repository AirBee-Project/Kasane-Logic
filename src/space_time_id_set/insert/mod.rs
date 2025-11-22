use std::process::id;

use crate::{
    bit_vec::BitVec,
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        Interval, SpaceTimeIdSet,
        insert::insert_main_dim::DimensionSelect,
        single::{
            convert_bitvec_f::convert_bitmask_f, convert_bitvec_xy::convert_bitmask_xy,
            convert_single_f::convert_f, convert_single_xy::convert_xy,
        },
    },
};
pub mod collect_other_dimension;
pub mod collect_top;
pub mod collect_under;
pub mod generate_index;
pub mod insert_main_dim;
pub mod search_under_count;
pub mod select_dimensions;
pub mod top_top_under;
pub mod uncheck_delete;
pub mod uncheck_insert;
pub mod uncheck_insert_dim;
pub mod under_under_top;

impl SpaceTimeIdSet {
    ///SpaceTimeIDSetに新規のIDを挿入する。
    /// 既存の範囲と重複がある場合は挿入時に調整が行われ、重複が排除される。
    pub fn insert(&mut self, id: SpaceTimeId) {
        let f_splited = convert_f(id.z, id.f);
        let x_splited = convert_xy(id.z, id.x);
        let y_splited = convert_xy(id.z, id.y);

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

        while !(f_encoded.is_empty() || x_encoded.is_empty() || y_encoded.is_empty()) {
            let (f_index, f_under_min_val) = {
                let (i, v) = f_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .expect("Internal error: f_encoded is empty");
                (i, (v.0, v.1.clone()))
            };

            let (x_index, x_under_min_val) = {
                let (i, v) = x_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .expect("Internal error: x_encoded is empty");
                (i, (v.0, v.1.clone()))
            };

            let (y_index, y_under_min_val) = {
                let (i, v) = y_encoded
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.0)
                    .expect("Internal error: y_encoded is empty");
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
                    DimensionSelect::F,
                );
            } else if min_under == x_under_min_val.0 {
                self.insert_main_dim(
                    &x_under_min_val.1,
                    &x_index,
                    &min_under,
                    &mut x_encoded,
                    &[&f_encoded, &y_encoded],
                    DimensionSelect::X,
                );
            } else {
                self.insert_main_dim(
                    &y_under_min_val.1,
                    &y_index,
                    &min_under,
                    &mut y_encoded,
                    &[&f_encoded, &x_encoded],
                    DimensionSelect::Y,
                );
            }
        }
    }
}
