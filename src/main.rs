use kasane_logic::{
    RangeId, SingleId,
    spatial_id::{ToFlexId, collection::set::memory::SetOnMemory},
};

fn main() {
    let mut set = SetOnMemory::new();
    let id = RangeId::new(5, [10, 11], [1, 5], [3, 10]).unwrap();
    let id2 = SingleId::new(0, 0, 0, 0).unwrap();
    println!("{}", id);

    set.insert(&id);
    set.insert(&id2);

    for range_id in set.flex_ids() {
        println!("{},", range_id.decode());
    }
}
