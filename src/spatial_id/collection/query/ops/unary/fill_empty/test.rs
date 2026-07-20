use crate::{SingleId, SpatialIdCollection, SpatialIdTable};
use alloc::collections::BTreeMap;

fn row(table: &SpatialIdTable<i32>) -> BTreeMap<u32, i32> {
    table
        .flat_single_ids()
        .map(|(sid, v)| (sid.x(), *v))
        .collect()
}

fn cell(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

#[test]
fn test_fill_empty_bounding_box() {
    let mut table = SpatialIdTable::new();
    table.insert(cell(100, 10).0, 10);
    table.insert(cell(102, 20).0, 20);

    // x=100(10) と x=102(20) の間にある x=101(空) を 0 で埋める
    let out = table.query().fill_empty(0).run().unwrap();
    let r = row(&out);

    assert_eq!(r.len(), 3);
    assert_eq!(r.get(&100), Some(&10));
    assert_eq!(r.get(&101), Some(&0));
    assert_eq!(r.get(&102), Some(&20));
}

#[test]
fn test_fill_empty_on_empty_table() {
    let table: SpatialIdTable<i32> = SpatialIdTable::new();
    let out = table.query().fill_empty(0).run().unwrap();
    assert_eq!(out.count(), 0);
}
