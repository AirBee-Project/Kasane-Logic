use kasane_logic::{FlexId, RangeId, SpatialIds};

fn main() {
    let id = RangeId::new(10, [4, 10], [3, 5], [4, 10]).unwrap();

    println!("{}", id);

    let a: Vec<FlexId> = id.flex_ids().collect();

    for flex_id in a {
        for single_id in flex_id.single_ids() {
            println!("{},", single_id);
        }
    }
}
