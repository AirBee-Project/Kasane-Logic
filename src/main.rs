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
        0,
        [Some(0), Some(0)],
        [Some(0), Some(0)],
        [Some(0), Some(0)],
        0,
        [None, None],
    )
    .unwrap();

    let id2 = SpaceTimeId::new(
        3,
        [Some(7), Some(7)],
        [Some(2), Some(7)],
        [Some(3), Some(4)],
        0,
        [None, None],
    )
    .unwrap();

    // let id1 = SpaceTimeId::random_z_max(5);
    // let id2 = SpaceTimeId::random_z_max(5);

    println!("{}", id1);
    println!(",{}", id2);
    println!("-----------");
    println!("-----------");

    set.insert(id2);
    set.insert(id1);

    for ele in set.get_all() {
        println!("{},", ele);
    }
}
