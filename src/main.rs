extern crate alloc;
use std::fs;
use std::str::FromStr;

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
use kasane_logic::spatial_id::collection::query::merge_policy::Max;
use kasane_logic::{SingleId, SpatialIdCollection, SpatialIdTable};

fn main() {
    let single_id = SingleId::from_str("20/0/931386/412903").unwrap();
    let single_id_2 = SingleId::from_str("20/5/931386/412903").unwrap();

    let mut table = SpatialIdTable::new();

    table.insert(single_id, 100);
    table.insert(single_id_2, 100);

    let table = table
        .query()
        .falloff_linear_x(24, 20, Max)
        .falloff_linear_f(24, 100, Max)
        .run()
        .unwrap();

    let json_string = serde_json::to_string_pretty(&table).unwrap();

    fs::write("output.json", json_string).unwrap();
}
