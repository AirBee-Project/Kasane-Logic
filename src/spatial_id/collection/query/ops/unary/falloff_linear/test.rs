use alloc::collections::BTreeMap;

use crate::spatial_id::collection::query::merge_policy::{Max, Sum};
use crate::{SingleId, Source, SpatialIdTable};

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

/// 単一セルの X falloff（半径2）。減衰は value*(r-|d|)/r。重なりが無いので Sum/Max は同値。
#[test]
fn falloff_x_single_cell() {
    let mut table = SpatialIdTable::new();
    let (id, v) = cell(100, 4);
    table.insert(id, v);

    let out = table
        .query()
        .falloff_linear_x(20, 2, Sum)
        .raw_run()
        .unwrap();
    let r = row(&out);

    // d=0:4, d=±1:2, d=±2:0
    assert_eq!(r.get(&98), Some(&0));
    assert_eq!(r.get(&99), Some(&2));
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&2));
    assert_eq!(r.get(&102), Some(&0));
}

/// 重なる2セルの X falloff を Sum で合成。重なったセルは両寄与の和になる。
#[test]
fn falloff_x_overlap_sum() {
    let mut table = SpatialIdTable::new();
    table.insert(cell(100, 4).0, 4);
    table.insert(cell(102, 4).0, 4);

    let out = table
        .query()
        .falloff_linear_x(20, 2, Sum)
        .raw_run()
        .unwrap();
    let r = row(&out);

    // A(x100) → x98:0 x99:2 x100:4 x101:2 x102:0
    // B(x102) → x100:0 x101:2 x102:4 x103:2 x104:0
    // Sum     → x98:0 x99:2 x100:4 x101:4 x102:4 x103:2 x104:0
    assert_eq!(r.get(&98), Some(&0));
    assert_eq!(r.get(&99), Some(&2));
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&4)); // 2+2
    assert_eq!(r.get(&102), Some(&4)); // 0+4
    assert_eq!(r.get(&103), Some(&2));
    assert_eq!(r.get(&104), Some(&0));
}

/// 同じ入力を Max で合成。重なりは最大値を取るため x101 が Sum と食い違う。
#[test]
fn falloff_x_overlap_max() {
    let mut table = SpatialIdTable::new();
    table.insert(cell(100, 4).0, 4);
    table.insert(cell(102, 4).0, 4);

    let out = table
        .query()
        .falloff_linear_x(20, 2, Max)
        .raw_run()
        .unwrap();
    let r = row(&out);

    // Max → x100:4 x101:2 x102:4（x101 は max(2,2)=2、Sum の 4 と異なる）
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&2));
    assert_eq!(r.get(&102), Some(&4));
}

/// 半径0の falloff は恒等（no-op）。
#[test]
fn falloff_x_radius_zero_is_noop() {
    let mut table = SpatialIdTable::new();
    table.insert(cell(100, 7).0, 7);

    let out = table
        .query()
        .falloff_linear_x(20, 0, Sum)
        .raw_run()
        .unwrap();
    let r = row(&out);

    assert_eq!(r.len(), 1);
    assert_eq!(r.get(&100), Some(&7));
}
