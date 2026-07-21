use alloc::collections::BTreeMap;

use crate::{SingleId, SpatialIdCollection, SpatialIdTable};

/// z=20, f=0, y=0 に固定した行から `x -> value` の対応を取り出す。
fn row(table: &SpatialIdTable<i32>) -> BTreeMap<u32, i32> {
    table
        .flat_single_ids()
        .map(|(sid, v)| (sid.x(), *v))
        .collect()
}

fn cell(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

/// X shift はセルを平行移動する（値は保つ）。
#[test]
fn shift_x_moves_cell() {
    let mut table = SpatialIdTable::new();
    table.insert(cell(100, 9).0, 9);
    table.insert(cell(200, 3).0, 3);

    let out = table.query().shift_x(20, 5).raw_run().unwrap();
    let r = row(&out);

    assert_eq!(r.len(), 2);
    assert_eq!(r.get(&105), Some(&9));
    assert_eq!(r.get(&205), Some(&3));
    assert_eq!(r.get(&100), None);
}
