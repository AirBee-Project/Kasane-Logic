use kasane_logic::{
    bit_vec::BitVec, space_time_id::SpaceTimeId, space_time_id_set::SpaceTimeIdSet,
};

fn main() {
    // let test1 = BitVec::from_vec(vec![0b10101011, 0b11000000]);
    // let test2 = BitVec::from_vec(vec![0b10101011, 0b11100000]);

    // let (start, end) = test1.under_prefix();
    // println!("START:{}", start);
    // println!("END  :{}", end);

    // if start < test2 {
    //     println!("{}<{}", start, test2);
    // }
    // if test2 < end {
    //     println!("{}<{}", test2, end);
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
