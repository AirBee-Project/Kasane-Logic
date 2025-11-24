use std::{collections::HashSet, fs::File};

use kasane_logic::{encode_id, encode_id_set::EncodeIDSet, space_time_id::SpaceTimeID};
use std::io::Write;
fn main() {
    let mut set = EncodeIDSet::new();

    let id1 = SpaceTimeID::new(5, [-1, 10], [2, 10], [5, 10], 10, [10, 40]).unwrap();
    let id2 = SpaceTimeID::new(4, [-1, 10], [2, 10], [5, 10], 10, [10, 40]).unwrap();
    let id3 = SpaceTimeID::new(1, [1, 1], [1, 1], [1, 1], 10, [10, 40]).unwrap();

    let mut file1 = File::create("output.txt").expect("cannot create file");

    let mut file2 = File::create("output_debug.txt").expect("cannot create file");

    println!("{},", id1);
    println!("{},", id2);
    println!("{},", id3);

    id1.to_encode().iter().for_each(|encode_id| {
        set.insert(encode_id.clone());
    });

    println!("-------------");

    id2.to_encode().iter().for_each(|encode_id| {
        set.insert(encode_id.clone());
    });
    println!("-------------");

    id3.to_encode().iter().for_each(|encode_id| {
        set.insert(encode_id.clone());
    });
    println!("-------------");

    for ele in set.iter() {
        writeln!(file1, "{},", ele).expect("cannot write to file");
    }

    writeln!(file2, "{:?},", set).expect("cannot write to file");

    println!("output.txt に書き出しました");
}
