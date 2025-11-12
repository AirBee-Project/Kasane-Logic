#[cfg(test)]
mod tests {
    use crate::{
        bit_vec::BitVec,
        space_time_id::SpaceTimeId,
        space_time_id_set::single::{
            convert_bitvec_f::convert_bitmask_f, convert_bitvec_xy::convert_bitmask_xy,
            convert_single_f::convert_f, convert_single_xy::convert_xy,
            invert_bitvec_f::invert_bitmask_f, invert_bitvec_xy::invert_bitmask_xy,
        },
    };

    #[test]
    fn convert_invert_test() {
        //1000個の時空間IDを生成
        let ids: Vec<SpaceTimeId> = (0..1000000)
            .map(|_| SpaceTimeId::random_z_max(10))
            .collect();

        for id in ids {
            let f_splited = convert_f(id.z, id.f);
            let x_splited = convert_xy(id.z, id.x);
            let y_splited = convert_xy(id.z, id.y);

            let f_encoded: Vec<BitVec> = f_splited
                .iter()
                .map(|(z, f)| convert_bitmask_f(*z, *f))
                .collect();
            let x_encoded: Vec<BitVec> = x_splited
                .iter()
                .map(|(z, x)| convert_bitmask_xy(*z, *x))
                .collect();
            let y_encoded: Vec<BitVec> = y_splited
                .iter()
                .map(|(z, y)| convert_bitmask_xy(*z, *y))
                .collect();

            let f_decoded: Vec<(u8, i64)> = f_encoded.iter().map(|v| invert_bitmask_f(v)).collect();
            let x_decoded: Vec<(u8, u64)> =
                x_encoded.iter().map(|v| invert_bitmask_xy(v)).collect();
            let y_decoded: Vec<(u8, u64)> =
                y_encoded.iter().map(|v| invert_bitmask_xy(v)).collect();

            //範囲が一致しているのかを検証
            for i in 0..f_encoded.len() {
                if f_splited[i] != f_decoded[i] {
                    println!("不一致を観測F");
                    println!("split : {:?}", f_splited[i]);
                    println!("decode : {:?}", f_decoded[i]);
                }
            }

            for i in 0..x_encoded.len() {
                if x_splited[i] != x_decoded[i] {
                    println!("不一致を観測X");
                    println!("split : {:?}", x_splited[i]);
                    println!("decode : {:?}", x_decoded[i]);
                }
            }

            for i in 0..y_encoded.len() {
                if y_splited[i] != y_decoded[i] {
                    println!("不一致を観測Y");
                    println!("split : {:?}", y_splited[i]);
                    println!("decode : {:?}", y_decoded[i]);
                }
            }
        }
    }
}
