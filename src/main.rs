use kasane_logic::{
    bit_vec::BitVec,
    space_time_id::SpaceTimeId,
    space_time_id_set::{
        SpaceTimeIdSet,
        single::{
            convert_bitvec_f::convert_bitmask_f, convert_bitvec_xy::convert_bitmask_xy,
            convert_single_f::convert_f, convert_single_xy::convert_xy,
            invert_bitvec_f::invert_bitmask_f, invert_bitvec_xy::invert_bitmask_xy,
        },
    },
};

fn main() {
    // let all = BitVec::from_vec(vec![0b10000000]);
    // let test1 = BitVec::from_vec(vec![0b10101011]);
    // let test2 = BitVec::from_vec(vec![0b10111011]);

    // let a = SpaceTimeIdSet::division(&all, &mut vec![test1, test2]);

    // for ele in a {
    //     println!("{}", ele);
    // }

    let mut set = SpaceTimeIdSet::new();

    let id1 = SpaceTimeId::new(
        4,
        [Some(3), Some(4)],
        [Some(3), Some(4)],
        [Some(3), Some(4)],
        0,
        [None, None],
    )
    .unwrap();

    let id2 = SpaceTimeId::new(
        5,
        [Some(7), Some(7)],
        [Some(8), Some(6)],
        [Some(6), Some(6)],
        0,
        [None, None],
    )
    .unwrap();
    println!("{},", id1);
    println!("{}", id2);
    println!("-----------");

    set.insert(id1);
    set.insert(id2);

    for ele in set.get_all() {
        println!("{},", ele);
    }
}

// for ele in convert_xy(id1.z, id1.x) {
//         println!("{}/-/{}/-,", ele.0, ele.1);
//         //println!("{}", convert_bitmask_f(ele.0, ele.1));
//     }

//     for ele in convert_xy(id2.z, id2.x) {
//         println!("{}/-/{}/-,", ele.0, ele.1);
//         //println!("{}", convert_bitmask_f(ele.0, ele.1));
//     }

//     println!("===========");

//     for ele in convert_xy(id1.z, id1.x) {
//         //println!("{}/{}/-/-,", ele.0, ele.1);
//         println!("{}", convert_bitmask_xy(ele.0, ele.1));
//     }

//     for ele in convert_xy(id2.z, id2.x) {
//         //println!("{}/{}/-/-,", ele.0, ele.1);
//         println!("{}", convert_bitmask_xy(ele.0, ele.1));
//     }

//     println!("===========");

//     for ele in convert_xy(id1.z, id1.x) {
//         let a = invert_bitmask_xy(&convert_bitmask_xy(ele.0, ele.1));

//         println!("{}/{}/-/-,", a.0, a.1);
//     }

//     for ele in convert_xy(id2.z, id2.x) {
//         let a = invert_bitmask_xy(&convert_bitmask_xy(ele.0, ele.1));

//         println!("{}/{}/-/-,", a.0, a.1);
//     }
