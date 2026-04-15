use kasane_logic::{FlexTree, RangeId, SingleId};

fn main() {
    let mut test = FlexTree::new();

    let id = RangeId::new(5, [-3, 10], [0, 9], [5, 10]).unwrap();
    let id2 = RangeId::new(4, [3, 6], [2, 2], [1, 9]).unwrap();
    let id3 = SingleId::new(2, 0, 1, 1).unwrap();

    test.insert(id);
    test.insert(id2);
    test.insert(id3);

    for ele in test.output() {
        let a = RangeId::from(ele);
        println!("{},", a);
    }
}
