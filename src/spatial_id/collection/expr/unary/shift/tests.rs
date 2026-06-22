use crate::{F_MAX, FlexId, ShiftOps, SingleId, SpatialIdSet, SpatialIdTable};

fn table_with(z: u8, f: i32, x: u32, y: u32) -> SpatialIdTable<bool> {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(z, f, x, y).unwrap(), true);
    table
}

/// 指定したセル（z, f, x, y）に値が存在するか。
fn present(table: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
    let cell = SingleId::new(z, f, x, y).unwrap();
    table.get(&cell).next().is_some()
}

// ---- F方向（鉛直・巡回なし） ----

#[test]
fn shift_f_up_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_f(25, 3).unwrap();

    assert!(!present(&result, 25, 0, 100, 100)); // 元の位置は空く
    assert!(present(&result, 25, 3, 100, 100)); // +3 へ移動
}

#[test]
fn shift_f_down_moves_cell() {
    let table = table_with(25, 10, 100, 100);
    let result = table.shift_f(25, -4).unwrap();

    assert!(!present(&result, 25, 10, 100, 100));
    assert!(present(&result, 25, 6, 100, 100));
}

#[test]
fn shift_f_out_of_range_is_error() {
    // z=25 の最上セルをさらに上へ動かすと範囲外。
    let table = table_with(25, F_MAX[25], 100, 100);
    assert!(table.shift_f(25, 1).is_err());
}

// ---- X方向（東西・巡回する） ----

#[test]
fn shift_x_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_x(25, 5).unwrap();

    assert!(!present(&result, 25, 0, 100, 100));
    assert!(present(&result, 25, 0, 105, 100));
}

#[test]
fn shift_x_wraps_around_seam() {
    // z=2 の最東セル x=3 を +1 すると、経度の境界を越えて x=0 へ巡回する。
    let table = table_with(2, 0, 3, 0);
    let result = table.shift_x(2, 1).unwrap();

    assert!(!present(&result, 2, 0, 3, 0));
    assert!(present(&result, 2, 0, 0, 0));
}

#[test]
fn shift_x_full_circle_returns_to_origin() {
    // z=2 では一周が4セル。+4 で元の位置へ戻る。
    let table = table_with(2, 0, 1, 0);
    let result = table.shift_x(2, 4).unwrap();

    assert!(present(&result, 2, 0, 1, 0));
}

#[test]
fn shift_x_splits_when_crossing_seam() {
    // X方向のみ粗い（z1）セルは z2 換算で2セル幅。+1 でちょうど境界をまたぎ、
    // RangeIdの巡回表現と同様に z2 の2セル（x=3 と x=0）へ分割される。
    let mut table = SpatialIdTable::new();
    let id = FlexId::new(2, 0, 1, 1, 2, 0).unwrap(); // x だけ z1 / index1（= z2 の 2,3）
    table.insert(id, true);

    let result = table.shift_x(2, 1).unwrap();

    assert!(present(&result, 2, 0, 3, 0)); // 元の 3 が残り
    assert!(present(&result, 2, 0, 0, 0)); // 4 が巡回して 0 へ
    assert!(!present(&result, 2, 0, 2, 0)); // 元の 2 は空く
}

// ---- Y方向（南北・巡回なし） ----

#[test]
fn shift_y_moves_cell() {
    let table = table_with(25, 0, 100, 100);
    let result = table.shift_y(25, -3).unwrap();

    assert!(!present(&result, 25, 0, 100, 100));
    assert!(present(&result, 25, 0, 100, 97));
}

#[test]
fn shift_y_below_zero_is_error() {
    // y=0 を下方向へ動かすと範囲外。
    let table = table_with(25, 0, 100, 0);
    assert!(table.shift_y(25, -1).is_err());
}

#[test]
fn shift_y_above_max_is_error() {
    // z=2 の最北セル y=3 を上へ動かすと範囲外。
    let table = table_with(2, 0, 0, 3);
    assert!(table.shift_y(2, 1).is_err());
}

// ---- Set でも同じ演算が使える（総称化の確認） ----

#[test]
fn shift_works_on_set() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(25, 0, 100, 100).unwrap());

    let result = set.shift_f(25, 3).unwrap();

    let moved = SingleId::new(25, 3, 100, 100).unwrap();
    let original = SingleId::new(25, 0, 100, 100).unwrap();
    assert!(result.get(&moved).next().is_some());
    assert!(result.get(&original).next().is_none());
}
