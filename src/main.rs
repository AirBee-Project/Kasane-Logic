use std::{collections::HashSet, fs::File};

use kasane_logic::{
    encode_id, encode_id_set::EncodeIDSet, function, id, point::Coordinate,
    space_time_id::SpaceTimeID,
};
use std::io::Write;
fn main() {
    let tokyo = Coordinate {
        latitude: 35.6809,
        longitude: 139.7673,
        altitude: 100.0,
    };
    let shinjuku = Coordinate {
        latitude: 35.6896,
        longitude: 139.6999,
        altitude: 100.0,
    };
    let result = function::line::line(25, tokyo, shinjuku);

    let mut file1 = File::create("output1.txt").expect("cannot create file");

    for ele in result.decode() {
        writeln!(file1, "{},", ele).expect("cannot write to file");
    }
}
