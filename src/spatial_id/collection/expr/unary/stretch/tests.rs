use crate::spatial_id::zoom_level::ZoomLevel;
use crate::{ConflictPolicy, FlexId, SingleId, SpatialIdSet, SpatialIdTable, StretchOps};

fn table_with(z: u8, f: i32, x: u32, y: u32) -> SpatialIdTable<bool> {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(z, f, x, y).unwrap(), true);
    table
}

fn present(table: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
    let cell = SingleId::new(z, f, x, y).unwrap();
    table.get(&cell).next().is_some()
}

// ---- F方向 ----

#[test]
fn stretch_f_keeps_original_and_fills_up() {
    // f=0 を +3 引き延ばす → 元の 0 と 1..=3 が埋まる（shift と違い 0 は残る）。
    let table = table_with(25, 0, 100, 100);
    let result = table.stretch_f(25, 3).unwrap();

    for f in 0..=3 {
        assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
    }
    assert!(!present(&result, 25, 4, 100, 100));
    assert!(!present(&result, 25, -1, 100, 100));
}

#[test]
fn stretch_f_negative_fills_down() {
    // f=10 を -2 引き延ばす → 8..=10 が埋まる。
    let table = table_with(25, 10, 100, 100);
    let result = table.stretch_f(25, -2).unwrap();

    for f in 8..=10 {
        assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
    }
    assert!(!present(&result, 25, 7, 100, 100));
    assert!(!present(&result, 25, 11, 100, 100));
}

#[test]
fn stretch_f_zero_is_identity() {
    let table = table_with(25, 4, 100, 100);
    let result = table.stretch_f(25, 0).unwrap();

    assert!(present(&result, 25, 4, 100, 100));
    assert!(!present(&result, 25, 3, 100, 100));
    assert!(!present(&result, 25, 5, 100, 100));
}

#[test]
fn stretch_f_out_of_range_is_error() {
    let table = table_with(
        25,
        unsafe { ZoomLevel::new_unchecked(25_u8) }.f_max(),
        100,
        100,
    );
    assert!(table.stretch_f(25, 1).is_err());
}

// ---- X方向（巡回） ----

#[test]
fn stretch_x_fills_range() {
    let table = table_with(25, 0, 100, 100);
    let result = table.stretch_x(25, 2).unwrap();

    for x in 100..=102 {
        assert!(present(&result, 25, 0, x, 100), "x={x} should be filled");
    }
    assert!(!present(&result, 25, 0, 103, 100));
}

#[test]
fn stretch_x_wraps_across_seam() {
    // z=2 の最東セル x=3 を +1 引き延ばす → x=3 と巡回先 x=0 が埋まる。
    let table = table_with(2, 0, 3, 0);
    let result = table.stretch_x(2, 1).unwrap();

    assert!(present(&result, 2, 0, 3, 0));
    assert!(present(&result, 2, 0, 0, 0));
    assert!(!present(&result, 2, 0, 1, 0));
}

#[test]
fn stretch_x_full_circle_covers_all() {
    // z=2 は一周4セル。+4 引き延ばすと全周（0..=3）を覆う。
    let table = table_with(2, 0, 1, 0);
    let result = table.stretch_x(2, 4).unwrap();

    for x in 0..=3 {
        assert!(present(&result, 2, 0, x, 0), "x={x} should be filled");
    }
}

// ---- Y方向（境界） ----

#[test]
fn stretch_y_negative_fills_down() {
    let table = table_with(25, 0, 100, 100);
    let result = table.stretch_y(25, -3).unwrap();

    for y in 97..=100 {
        assert!(present(&result, 25, 0, 100, y), "y={y} should be filled");
    }
    assert!(!present(&result, 25, 0, 100, 96));
}

#[test]
fn stretch_y_out_of_range_is_error() {
    let table = table_with(25, 0, 100, 0);
    assert!(table.stretch_y(25, -1).is_err());
}

// ---- 総称化の確認 ----

#[test]
fn stretch_works_on_set() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(25, 0, 100, 100).unwrap());

    let result = set.stretch_f(25, 2).unwrap();

    for f in 0..=2 {
        let cell = SingleId::new(25, f, 100, 100).unwrap();
        assert!(result.get(&cell).next().is_some(), "f={f} should be filled");
    }
}

// 値が重なったときの衝突解決（値付き Table）。
#[test]
fn stretch_resolves_overlap_by_policy() {
    // f=0 に値1、f=2 に値9。両方 +2 引き延ばすと f=2 で重なる（1 由来 vs 9 由来）。
    let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
    table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

    let value_at = |t: &SpatialIdTable<u8>, f: i32| {
        let cell = SingleId::new(25, f, 100, 100).unwrap();
        t.get(&cell).next().map(|(_, v)| *v)
    };

    // Max: 重なった f=2 は max(1 の伸長, 9) = 9。
    let by_max = table.stretch_f_with(25, 2, ConflictPolicy::Max).unwrap();
    assert_eq!(value_at(&by_max, 2), Some(9));

    // Min: 重なった f=2 は min(...) = 1。
    let by_min = table.stretch_f_with(25, 2, ConflictPolicy::Min).unwrap();
    assert_eq!(value_at(&by_min, 2), Some(1));
}

// FlexId を直接使う混在ズームの確認（X だけ粗いセルの引き延ばし）。
#[test]
fn stretch_x_on_coarse_cell() {
    let mut table = SpatialIdTable::new();
    table.insert(FlexId::new(2, 0, 1, 1, 2, 0).unwrap(), true); // x は z1 / index1（= z2 の 2,3）
    let result = table.stretch_x(2, 1).unwrap();

    // 元の 2,3 に加え、+1 で 4→0（巡回）まで埋まる。
    for x in [0u32, 2, 3] {
        assert!(present(&result, 2, 0, x, 0), "x={x} should be filled");
    }
    assert!(!present(&result, 2, 0, 1, 0));
}
