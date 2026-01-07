use kasane_logic::spatial_id::{SpatialId, single::SingleId};

fn main() {
    let mut id = SingleId::new(3, 4, 3, 2).unwrap();

    println!("{}", id);
}
