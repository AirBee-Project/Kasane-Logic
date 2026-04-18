use kasane_logic::{FlexTreeSet, FlexTreeTable, IntoFlexIds, RangeId, SingleId};

fn main() {
    let mut table = FlexTreeTable::new();

    let id1 = RangeId::new(5, [-3, 10], [0, 9], [5, 10]).unwrap();
    let id2 = RangeId::new(4, [3, 6], [2, 2], [1, 9]).unwrap();
    let id3 = SingleId::new(2, 0, 1, 1).unwrap();

    table.insert(id1, "Neko".to_string());
}
