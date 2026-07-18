extern crate alloc;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::str::FromStr;

use std::io::Write;

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
use kasane_logic::{RangeId, SpatialIdCollection, SpatialIdSet};

fn main() {
    let mut raw_ids = fs::read_to_string("sample/bldg1.txt").unwrap();
    raw_ids.retain(|c| !c.is_whitespace());

    let mut set = SpatialIdSet::new();

    for raw_range_id in raw_ids.split(",") {
        let range_id = match RangeId::from_str(raw_range_id) {
            Ok(v) => v,
            Err(_) => {
                continue;
            }
        };
        set.insert(range_id);
    }

    // 1. 書き込み用のファイルを作成
    let file = File::create("output.txt").unwrap();

    // 2. 高速化のためにBufWriterでラップする
    let mut writer = BufWriter::new(file);

    let set = set.query().shift_x(25, 10).run().unwrap();

    for ele in set.iter() {
        writeln!(writer, "{},", RangeId::from(ele)).unwrap();
    }

    writer.flush().unwrap();

    println!("ファイルの書き込みが完了しました。");
    println!("{}", set.iter().count())
}
