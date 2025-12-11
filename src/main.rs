use std::collections::HashMap;

use kasane_logic::{
    geometry::point::coordinate::Coordinate,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, XY_MAX},
        range::RangeID,
        single::SingleID,
    },
};

fn main() {
    let mut id = SingleID::new(4, 6, 9, 14).unwrap();

    let center: [Coordinate; 8] = id.vertices();
    println!("{:?}", center);
}
