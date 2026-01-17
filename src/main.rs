use kasane_logic::spatial_id::{map::SpatialIdSet, range::RangeId, single::SingleId};

fn main() {
    let mut set = SpatialIdSet::new();

    let id = RangeId::new(5, [-10, 13], [1, 13], [1, 10]).unwrap();
    let id2 = SingleId::new(2, 1, 1, 1).unwrap();

    println!("{}", id);
    println!("{}", id2);

    set.insert(&id);
    set.insert(&id2);

    for ele in set.iter() {
        println!("{},", ele);
    }
}
