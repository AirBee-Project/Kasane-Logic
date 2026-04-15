use kasane_logic::{FlexTreeCore, FlexTreeSet, RangeId, SingleId};

fn main() {
    let mut test = FlexTreeSet::new();

    let id = RangeId::new(5, [-3, 10], [0, 9], [5, 10]).unwrap();
    let id2 = RangeId::new(4, [3, 6], [2, 2], [1, 9]).unwrap();
    let id3 = SingleId::new(2, 0, 1, 1).unwrap();

    println!("{}", id);
    println!("{},", id2);
    println!("{},", id3);

    println!("======");

    test.insert(id);
    test.insert(id2);

    test.remove(&id3);

    for ele in test.iter() {
        let a = RangeId::from(ele);
        println!("{},", a);
    }
}
