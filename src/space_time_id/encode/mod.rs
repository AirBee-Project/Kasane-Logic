use crate::{
    bit_vec::BitVec,
    encode_id::EncodeID,
    space_time_id::{
        SpaceTimeID,
        encode::{
            segment::{segment_f, segment_xy},
            to_bitvec::{to_bitvec_f, to_bitvec_xy},
        },
    },
};

pub mod segment;
pub mod to_bitvec;

use itertools::iproduct;

impl SpaceTimeID {
    pub fn to_encode(&self) -> Vec<EncodeID> {
        let f_splited = segment_f(self.z, self.f);
        let x_splited = segment_xy(self.z, self.x);
        let y_splited = segment_xy(self.z, self.y);

        iproduct!(&f_splited, &x_splited, &y_splited)
            .map(|((z_f, f), (z_x, x), (z_y, y))| EncodeID {
                f: to_bitvec_f(*z_f, *f),
                x: to_bitvec_xy(*z_x, *x),
                y: to_bitvec_xy(*z_y, *y),
            })
            .collect()
    }
}
