use std::fs::File;

use kasane_logic::{space_time_id::SpaceTimeId, space_time_id_set::SpaceTimeIdSet};
use std::io::Write;
fn main() {
    let mut set = SpaceTimeIdSet::new();

    let id1 = SpaceTimeId::new(
        5,
        [Some(-1), Some(10)],
        [Some(2), Some(10)],
        [Some(5), Some(10)],
        10,
        [Some(10), Some(40)],
    )
    .unwrap();
    let id2 = SpaceTimeId::new(
        4,
        [Some(-1), Some(10)],
        [Some(2), Some(10)],
        [Some(5), Some(10)],
        10,
        [Some(10), Some(40)],
    )
    .unwrap();

    let id3 = SpaceTimeId::new(
        1,
        [Some(1), Some(1)],
        [Some(1), Some(1)],
        [Some(1), Some(1)],
        10,
        [Some(10), Some(40)],
    )
    .unwrap();

    let mut file = File::create("output.txt").expect("cannot create file");

    set.insert(id1);
    set.insert(id2);
    set.insert(id3);

    println!("{},", id1);
    println!("{},", id2);
    println!("{},", id3);

    for ele in set.get_all() {
        writeln!(file, "{},", ele).expect("cannot write to file");
    }

    println!("output.txt に書き出しました");
}
