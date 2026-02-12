use std::collections::btree_map::Range;

use kasane_logic::{RangeId, SetOnMemory, SingleId, TableOnMemory};

fn main() {
    let mut table1: TableOnMemory<String> = TableOnMemory::new();

    let id1 = RangeId::new(5, [9, 15], [10, 23], [10, 13]).unwrap();

    println!("{}", id1);

    table1.insert(&id1, &"neko".to_string());

    for ele in table1.range_ids() {
        println!("{},", ele.0);
    }
}
