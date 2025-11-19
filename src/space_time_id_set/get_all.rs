use crate::{
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        SpaceTimeIdSet,
        single::{invert_bitvec_f::invert_bitmask_f, invert_bitvec_xy::invert_bitmask_xy},
    },
};

impl SpaceTimeIdSet {
    ///SpaceTimeIDSet内部の全てのIDを取得する
    pub fn get_all(&self) -> Vec<SpaceTimeId> {
        let mut result = vec![];
        for (index, reverse) in &self.reverse {
            let (f_z, f_v) = invert_bitmask_f(&reverse.f);
            let (x_z, x_v) = invert_bitmask_xy(&reverse.x);
            let (y_z, y_v) = invert_bitmask_xy(&reverse.y);

            let max_z = f_z.max(x_z).max(y_z);

            let f = if max_z == f_z {
                [f_v, f_v]
            } else {
                let k = 2_i64.pow((max_z - f_z).into());
                [f_v * k, (f_v + 1) * k - 1]
            };

            let x = if max_z == x_z {
                [x_v, x_v]
            } else {
                let k = 2_u64.pow((max_z - x_z).into());
                [x_v * k, (x_v + 1) * k - 1]
            };

            let y = if max_z == y_z {
                [y_v, y_v]
            } else {
                let k = 2_u64.pow((max_z - y_z).into());
                [y_v * k, (y_v + 1) * k - 1]
            };

            result.push(SpaceTimeId {
                z: max_z,
                f,
                x,
                y,
                i: 0,
                t: [0, u64::MAX],
            });
        }

        result
    }
}
