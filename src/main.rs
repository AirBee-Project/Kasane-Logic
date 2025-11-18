use kasane_logic::{
    bit_vec::BitVec, space_time_id::SpaceTimeId, space_time_id_set::SpaceTimeIdSet,
};

fn main() {
    // let test1 = BitVec::from_vec(vec![0b11101111]);

    // let (start, end) = test1.under_prefix();
    // println!("START:{}", start);
    // println!("END  :{}", end);

    let mut set = SpaceTimeIdSet::new();

    let id1 = SpaceTimeId::new(
        5,
        [Some(31), Some(31)],
        [Some(14), Some(25)],
        [Some(0), Some(13)],
        0,
        [None, None],
    )
    .unwrap();

    let id2 = SpaceTimeId::new(
        3,
        [Some(7), Some(7)],
        [Some(4), Some(6)],
        [Some(1), Some(7)],
        0,
        [None, None],
    )
    .unwrap();
    println!("{}", id1);
    println!(",{}", id2);
    println!("-----------");

    set.insert(id1);

    println!("-----------");

    set.insert(id2);

    for ele in set.get_all() {
        println!("{},", ele);
    }
}
