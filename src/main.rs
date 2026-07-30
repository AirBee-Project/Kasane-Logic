use kasane_logic::{
    Source, SpatialIdTable,
    merge_policy::{Average, Max},
};
use std::fs;

fn main() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();

    let risk = bldg_risk
        .query()
        .zoom_out(22, Average)
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_y(25, 3, Max)
        .raw_run_table()
        .unwrap();

    let json_string = serde_json::to_string(&risk).unwrap();

    fs::write("output.json", json_string).unwrap();
}
