use kasane_logic::{FlexId, IntoFlexIds, RangeId, VBitTree};

fn main() {
    let mut test = VBitTree::new();

    let id = RangeId::new(5, [-3, 10], [0, 9], [5, 10]).unwrap();

    println!("{}", id);

    let id: Vec<FlexId> = id.into_flex_ids().collect();

    for ele in id {
        test.insert(ele);
    }

    for ele in test.output() {
        let a = RangeId::from(ele);
        println!("{},", a);
    }
}
