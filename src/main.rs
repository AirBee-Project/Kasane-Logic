use kasane_logic::{
    RangeId,
    spatial_id::{ToFlexId, collection::set::memory::SetOnMemory},
};
use rayon::range;

fn main() {
    let mut set = SetOnMemory::new();
    let id = RangeId::new(5, [10, 11], [1, 5], [3, 10]).unwrap();
    println!("{}", id);

    set.insert(&id);

    for range_id in id.to_flex_id() {
        println!("{},", range_id.decode());
    }
}
