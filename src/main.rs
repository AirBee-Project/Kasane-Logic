use kasane_logic::{FlexId, RangeId, SpatialIdCollection, SpatialIdTable, merge_policy::Max};
use std::{fs, str::FromStr};

fn main() {
    let bldg_risk: SpatialIdTable<u32> =
        serde_json::from_str(&fs::read_to_string("sample/bldg_risk.json").unwrap()).unwrap();

    let clip = RangeId::from_str("23/0:31/7450128:7450228/3301266:3301523").unwrap();

    println!("{}", bldg_risk.bounding_box().unwrap());

    let risk: Vec<(FlexId, u32)> = bldg_risk
        .query()
        .falloff_linear_x(25, 3, Max)
        .falloff_linear_y(25, 3, Max)
        .lazy()
        .get(clip)
        .unwrap()
        .collect();

    let mut result = SpatialIdTable::new();

    for ele in risk {
        result.insert(ele.0, ele.1);
    }

    let json_string = serde_json::to_string(&result).unwrap();

    fs::write("output.json", json_string).unwrap();
}
