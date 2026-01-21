use kasane_logic::{RangeId, SetOnMemory, SingleId};

fn main() {
    let mut set = SetOnMemory::new();
    let mut set2 = SetOnMemory::new();

    let id = RangeId::new(5, [7, 11], [1, 5], [3, 10]).unwrap();
    let id2 = SingleId::new(3, 2, 1, 2).unwrap();
    let id3 = SingleId::new(3, 2, 0, 1).unwrap();
    set2.insert(&id3);

    println!("{}", id);
    println!("{}", id2);
    println!("{}", id3);

    set.insert(&id);
    set.insert(&id2);

    let set3 = set.intersection(&set2);

    for range_id in set3.flex_ids() {
        println!("{},", range_id.decode());
    }
}
