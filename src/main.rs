use kasane_logic::{RangeId, SetOnMemory, SingleId};

fn main() {
    let mut set1 = SetOnMemory::new();
    let mut set2 = SetOnMemory::new();

    {
        let id1 = RangeId::new(5, [3, 4], [3, 3], [1, 4]).unwrap();
        let id2 = SingleId::new(4, 2, 1, 1).unwrap();
        set1.insert(&id1);
        set1.insert(&id2);
    }

    {
        let id1 = SingleId::new(3, 1, 0, 0).unwrap();
        set2.insert(&id1);
    }

    let set3 = set1.intersection(&set2);

    for range_id in set3.range_ids() {
        println!("{},", range_id);
    }
}
