use crate::{ConflictPolicy, DiffuseOps, SingleId, SpatialIdTable};

fn table_u8(z: u8, f: i32, x: u32, y: u32, value: u8) -> SpatialIdTable<u8> {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(z, f, x, y).unwrap(), value);
    table
}

fn value_at(table: &SpatialIdTable<u8>, z: u8, f: i32, x: u32, y: u32) -> Option<u8> {
    let cell = SingleId::new(z, f, x, y).unwrap();
    table.get(&cell).next().map(|(_, v)| *v)
}

// ---- X方向（巡回） ----

#[test]
fn diffuse_x_decays_both_directions_and_keeps_original() {
    // 値200・減衰60 → 段階的に 140, 80, 20 と両方向へ広がる。20<=60 で打ち切り。
    let table = table_u8(25, 0, 100, 100, 200);
    let result = table.diffuse_x(25, 5, 60).unwrap();

    assert_eq!(value_at(&result, 25, 0, 100, 100), Some(200)); // 元の値は残る
    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(140));
    assert_eq!(value_at(&result, 25, 0, 99, 100), Some(140));
    assert_eq!(value_at(&result, 25, 0, 102, 100), Some(80));
    assert_eq!(value_at(&result, 25, 0, 98, 100), Some(80));
    assert_eq!(value_at(&result, 25, 0, 103, 100), Some(20));
    assert_eq!(value_at(&result, 25, 0, 97, 100), Some(20));
    // distance=5 でも減衰が尽きて 4 ステップ目以降は書かれない。
    assert_eq!(value_at(&result, 25, 0, 104, 100), None);
    assert_eq!(value_at(&result, 25, 0, 96, 100), None);
}

#[test]
fn diffuse_x_zero_decay_is_constant_halo() {
    // 減衰0 → distance 分だけ同じ値で広がる。
    let table = table_u8(25, 0, 100, 100, 50);
    let result = table.diffuse_x(25, 2, 0).unwrap();

    for x in 98..=102 {
        assert_eq!(value_at(&result, 25, 0, x, 100), Some(50), "x={x}");
    }
    assert_eq!(value_at(&result, 25, 0, 103, 100), None);
}

#[test]
fn diffuse_x_wraps_across_seam() {
    // z=2 は一周4セル。最東 x=3 から +1 で x=0 へ巡回。
    let table = table_u8(2, 0, 3, 0, 200);
    let result = table.diffuse_x(2, 1, 60).unwrap();

    assert_eq!(value_at(&result, 2, 0, 3, 0), Some(200));
    assert_eq!(value_at(&result, 2, 0, 0, 0), Some(140)); // +1 で巡回
    assert_eq!(value_at(&result, 2, 0, 2, 0), Some(140)); // -1
}

#[test]
fn diffuse_x_overlap_takes_max() {
    // 2つの発生源のハローが重なる領域は Max で高い方を採用。
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 200u8);
    table.insert(SingleId::new(25, 0, 104, 100).unwrap(), 200u8);
    let result = table.diffuse_x(25, 3, 60).unwrap();

    // x=102: 100から+2(80) と 104から-2(80) → 80
    assert_eq!(value_at(&result, 25, 0, 102, 100), Some(80));
    // x=101: 100から+1(140) と 104から-3(20) → max=140
    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(140));
    // x=103: 100から+3(20) と 104から-1(140) → max=140
    assert_eq!(value_at(&result, 25, 0, 103, 100), Some(140));
}

#[test]
fn diffuse_x_overlap_with_min_policy() {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 200u8);
    table.insert(SingleId::new(25, 0, 104, 100).unwrap(), 200u8);
    let result = table
        .diffuse_x_with(25, 3, 60, ConflictPolicy::Min)
        .unwrap();

    // x=101: min(140, 20) = 20
    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(20));
}

// ---- Y方向（境界クリップ） ----

#[test]
fn diffuse_y_clips_at_boundary() {
    // y=0 から下方向(-)は範囲外 → エラーにせずクリップ。上方向(+)のみ広がる。
    let table = table_u8(25, 0, 100, 0, 200);
    let result = table.diffuse_y(25, 2, 60).unwrap();

    assert_eq!(value_at(&result, 25, 0, 100, 0), Some(200)); // 元の値
    assert_eq!(value_at(&result, 25, 0, 100, 1), Some(140)); // +1
    assert_eq!(value_at(&result, 25, 0, 100, 2), Some(80)); // +2
}

// ---- F方向 ----

#[test]
fn diffuse_f_decays_both_directions() {
    let table = table_u8(25, 10, 100, 100, 200);
    let result = table.diffuse_f(25, 2, 60).unwrap();

    assert_eq!(value_at(&result, 25, 10, 100, 100), Some(200));
    assert_eq!(value_at(&result, 25, 11, 100, 100), Some(140));
    assert_eq!(value_at(&result, 25, 9, 100, 100), Some(140));
    assert_eq!(value_at(&result, 25, 12, 100, 100), Some(80));
    assert_eq!(value_at(&result, 25, 8, 100, 100), Some(80));
}
