use kasane_logic::{Coordinate, RangeId, SingleId, SpatialIdMap, SpatialIdSet};

fn main() {
    let mut set = SpatialIdSet::new();

    let id = RangeId::new(5, [-10, 13], [1, 13], [1, 10]).unwrap();
    let id2 = SingleId::new(2, 1, 1, 1).unwrap();
    let id3 = SingleId::new(3, 3, 3, 1).unwrap();
    let id4 = SingleId::new(3, 2, 3, 1).unwrap();
    let id4 = SingleId::new(3, 1, 3, 1).unwrap();

    println!("{}", id);
    println!("{}", id2);

    set.insert(&id);
    set.insert(&id2);
    set.insert(&id3);
    set.insert(&id4);

    let diff = SingleId::new(3, 3, 0, 2).unwrap();
    let mut set2 = SpatialSet::new();
    set2.insert(&diff);

    let set3 = set.intersection(&set2);

    for ele in set3.iter() {
        println!("{},", ele);
    }
}

// fn main() {
//     let segments: Vec<_> = Segment::<u32>::new(3, [5, 5]).collect();
//     let segment = segments.first().unwrap().clone();

//     let encode = EncodeSegment::from(segment);

//     println!("{}", encode);
//     println!("{}", encode.descendant_range_end());
// }
