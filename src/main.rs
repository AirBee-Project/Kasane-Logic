use kasane_logic::{RangeId, SetOnMemory, SingleId};

fn main() {
    let mut set = SetOnMemory::default();
    let id1 = RangeId::new(5, [-7, 11], [1, 5], [5, 30]).unwrap();
    println!("START");

    set.insert(&id1);
    let id2 = RangeId::new(3, [2, 2], [1, 5], [2, 2]).unwrap();
    println!("START");

    set.insert(&id2);
    println!("START");

    for ele in set.range_ids() {
        println!("{},", ele);
    }
}
