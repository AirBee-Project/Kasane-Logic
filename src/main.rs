use std::collections::btree_map::Range;

use kasane_logic::{RangeId, SetOnMemory, SingleId, TableOnMemory};

fn main() {
    let mut table1: TableOnMemory<String> = TableOnMemory::new();

    let id1 = SingleId::new(5, 10, 1, 3).unwrap();

    table1.insert(&id1, &"neko".to_string());

    for ele in table1.range_ids() {
        println!("{},", ele.0);
        println!("{},", ele.1);
    }
}
