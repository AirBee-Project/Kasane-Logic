use std::collections::HashSet;

use crate::{
    bit_vec::BitVec,
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        Index, ReverseInfo, SpaceTimeIdSet,
        insert::insert_main_dim::DimensionSelect,
        single::{invert_bitvec_f::invert_bitmask_f, invert_bitvec_xy::invert_bitmask_xy},
    },
};

impl SpaceTimeIdSet {
    ///相手を切断する
    pub fn top_top_under(
        &mut self,
        target_index: Index,
        target_bit: BitVec,
        target_dim: DimensionSelect,
        need_delete: &mut HashSet<Index>,
        need_insert: &mut HashSet<ReverseInfo>,
    ) {
        // println!("top_top_under");

        // println!("{:?}軸で分割を行う", target_dim);

        // reverseから必要なデータをcloneして借用を解放
        let reverse = self.reverse.get(&target_index).unwrap();

        // {
        //     let (f_z, f_v) = invert_bitmask_f(&reverse.f);
        //     let (x_z, x_v) = invert_bitmask_xy(&reverse.x);
        //     let (y_z, y_v) = invert_bitmask_xy(&reverse.y);

        //     let max_z = f_z.max(x_z).max(y_z);

        //     let f = if max_z == f_z {
        //         [f_v, f_v]
        //     } else {
        //         let k = 2_i64.pow((max_z - f_z).into());
        //         [f_v * k, (f_v + 1) * k - 1]
        //     };

        //     let x = if max_z == x_z {
        //         [x_v, x_v]
        //     } else {
        //         let k = 2_u64.pow((max_z - x_z).into());
        //         [x_v * k, (x_v + 1) * k - 1]
        //     };

        //     let y = if max_z == y_z {
        //         [y_v, y_v]
        //     } else {
        //         let k = 2_u64.pow((max_z - y_z).into());
        //         [y_v * k, (y_v + 1) * k - 1]
        //     };

        //     println!(
        //         "{}",
        //         SpaceTimeId {
        //             z: max_z,
        //             f,
        //             x,
        //             y,
        //             i: 0,
        //             t: [0, u64::MAX],
        //         }
        //     );
        // }

        let top = match target_dim {
            DimensionSelect::F => reverse.f.clone(),
            DimensionSelect::X => reverse.x.clone(),
            DimensionSelect::Y => reverse.y.clone(),
        };

        let splited = BitVec::division(top, vec![target_bit]);

        // for ele in &splited {
        //     let tmp = invert_bitmask_xy(&ele);
        //     println!("{}/-/-/{}", tmp.0, tmp.1);
        // }

        // ここでreverseのフィールドを個別にclone
        let reverse_f = reverse.f.clone();
        let reverse_x = reverse.x.clone();
        let reverse_y = reverse.y.clone();

        for single in splited {
            match target_dim {
                DimensionSelect::F => need_insert.insert(ReverseInfo {
                    f: single,
                    x: reverse_x.clone(),
                    y: reverse_y.clone(),
                }),
                DimensionSelect::X => need_insert.insert(ReverseInfo {
                    f: reverse_f.clone(),
                    x: single,
                    y: reverse_y.clone(),
                }),
                DimensionSelect::Y => need_insert.insert(ReverseInfo {
                    f: reverse_f.clone(),
                    x: reverse_x.clone(),
                    y: single,
                }),
            };
        }

        need_delete.insert(target_index);
    }
}
