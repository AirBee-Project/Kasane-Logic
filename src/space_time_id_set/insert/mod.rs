use crate::{
    bit_vec::BitVec,
    space_time_id::{
        SpaceTimeID,
        encode::{
            segment::{segment_f, segment_xy},
            to_bitvec::{to_bitvec_f, to_bitvec_xy},
        },
    },
    space_time_id_set::{SpaceTimeIDSet, insert::select_dimensions::DimensionSelect},
};
pub mod collect_ancestors;
pub mod collect_descendants;
pub mod collect_other_dimension;
pub mod generate_index;
pub mod insert_main_dim;
pub mod select_dimensions;
pub mod split_other;
pub mod split_self;
pub mod uncheck_delete;
pub mod uncheck_insert;

impl SpaceTimeIDSet {
    ///SpaceTimeIDSetに新規のIDを挿入する。
    /// 既存の範囲と重複がある場合は挿入時に調整が行われ、重複が排除される。
    pub fn insert(&mut self, id: SpaceTimeID) {
        let f_splited = segment_f(id.z, id.f);
        let x_splited = segment_xy(id.z, id.x);
        let y_splited = segment_xy(id.z, id.y);

        let mut f_encoded: Vec<(usize, BitVec)> = f_splited
            .iter()
            .map(|(z, f)| {
                let bit_vec = to_bitvec_f(*z, *f);
                let count = self.f.get(&bit_vec).map_or(0, |v| v.count);
                (count, bit_vec)
            })
            .collect();

        let mut x_encoded: Vec<(usize, BitVec)> = x_splited
            .iter()
            .map(|(z, x)| {
                let bit_vec = to_bitvec_xy(*z, *x);
                let count = self.x.get(&bit_vec).map_or(0, |v| v.count);
                (count, bit_vec)
            })
            .collect();

        let mut y_encoded: Vec<(usize, BitVec)> = y_splited
            .iter()
            .map(|(z, y)| {
                let bit_vec = to_bitvec_xy(*z, *y);
                let count = self.y.get(&bit_vec).map_or(0, |v| v.count);
                (count, bit_vec)
            })
            .collect();

        println!("{:?}", y_encoded);

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
                    &x_encoded,
                    &y_encoded,
                    DimensionSelect::F,
                );
            } else if min_under == x_under_min_val.0 {
                self.insert_main_dim(
                    &x_under_min_val.1,
                    &x_index,
                    &min_under,
                    &mut x_encoded,
                    &f_encoded,
                    &y_encoded,
                    DimensionSelect::X,
                );
            } else {
                self.insert_main_dim(
                    &y_under_min_val.1,
                    &y_index,
                    &min_under,
                    &mut y_encoded,
                    &f_encoded,
                    &x_encoded,
                    DimensionSelect::Y,
                );
            }
        }
    }
}
