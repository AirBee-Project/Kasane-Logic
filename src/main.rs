use kasane_logic::{FlexTreeSet, IntoFlexIds, RangeId, SingleId};

fn main() {
    let mut set1 = FlexTreeSet::new();
    let mut set2 = FlexTreeSet::new();

    let id1 = RangeId::new(5, [-3, 10], [0, 9], [5, 10]).unwrap();
    let id2 = RangeId::new(4, [3, 6], [2, 2], [1, 9]).unwrap();
    let id3 = SingleId::new(2, 0, 1, 1).unwrap();
    println!("{},", id1);
    println!("{},", id2);
    println!("{},", id3);
    println!("=====");

    set1.insert(id1);

    set2.insert(id2);
    set2.insert(id3);

    let set3 = set1 & set2;

    for flex_id in set3.into_flex_ids() {
        println!("{},", RangeId::from(flex_id))
    }
}
