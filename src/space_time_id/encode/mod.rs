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

impl SpaceTimeID {
    pub fn to_encode(&self) -> Vec<EncodeID> {
        let mut result = vec![];

        //セグメントに区切る
        let f_splited = segment_f(self.z, self.f);
        let x_splited = segment_xy(self.z, self.x);
        let y_splited = segment_xy(self.z, self.y);

        for (z_f, f) in &f_splited {
            let f_bitvec = to_bitvec_f(*z_f, *f);
            for (z_x, x) in &x_splited {
                let x_bitvec = to_bitvec_xy(*z_x, *x);
                for (z_y, y) in &y_splited {
                    let y_bitvec = to_bitvec_xy(*z_y, *y);
                    result.push(EncodeID {
                        f: f_bitvec.clone(),
                        x: x_bitvec.clone(),
                        y: y_bitvec,
                    });
                }
            }
        }

        result
    }
}
