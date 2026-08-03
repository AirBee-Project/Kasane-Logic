use alloc::collections::BTreeMap;

use crate::{SingleId, Source, SpatialIdTable};

/// z=20, f=0, y=0 に固定した行から `x -> value` の対応を取り出す。
fn row(table: &SpatialIdTable<i32>) -> BTreeMap<u32, i32> {
    table
        .flat_single_ids()
        .map(|(sid, v)| (sid.x(), *v))
        .collect()
}

fn time_segment(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

/// X shift はSegmentを平行移動する（値は保つ）。
#[test]
fn shift_x_moves_segment() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 9).0, 9);
    table.insert(time_segment(200, 3).0, 3);

    let out = table.query().shift_x(20, 5).raw_run_table().unwrap();
    let r = row(&out);

    assert_eq!(r.len(), 2);
    assert_eq!(r.get(&105), Some(&9));
    assert_eq!(r.get(&205), Some(&3));
    assert_eq!(r.get(&100), None);
}
