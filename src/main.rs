use kasane_logic::{FlexId, FlexIds, RangeId, VBitTree};

fn main() {
    let mut test = VBitTree::new();

    let id: Vec<FlexId> = RangeId::new(4, [-3, 10], [8, 9], [5, 10])
        .unwrap()
        .flex_ids()
        .collect();

    for ele in id {
        test.insert(ele);
    }

    for ele in test.output() {
        let a = RangeId::from(ele);
        println!("{},", a);
    }
}
