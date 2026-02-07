use kasane_logic::{RangeId, SetOnMemory};

fn main() {
    let mut set = SetOnMemory::new();
    let id1 = RangeId::new(5, [3, 4], [3, 3], [1, 4]).unwrap();
    set.insert(&id1);

    for range_id in set.range_ids() {
        println!("{},", range_id);
    }

    println!("OK");
}
