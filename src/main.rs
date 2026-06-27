extern crate alloc;
use std::str::FromStr;
use std::{fs, mem};

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
use kasane_logic::{RangeId, SpatialIdSet};

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

    println!("x size: {} bytes", mem::size_of_val(&set));
}
