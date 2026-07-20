extern crate alloc;
use std::fs;

#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;
use kasane_logic::spatial_id::collection::query::merge_policy::{Average, Max};
use kasane_logic::{SpatialIdCollection, SpatialIdTable};

fn main() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();

    let risk = bldg_risk
        .query()
        .zoom_out(22, Average)
        .extrude_f(25, 0, 50, Max)
        .falloff_linear_x(25, 10, Max)
        .falloff_linear_y(25, 10, Max)
        .falloff_linear_f(25, 5, Max)
        .fill_empty(0)
        .run()
        .unwrap();

    let json_string = serde_json::to_string_pretty(&risk).unwrap();

    fs::write("output.json", json_string).unwrap();
}
