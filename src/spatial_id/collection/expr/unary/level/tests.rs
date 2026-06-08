use crate::{ConflictPolicy, FlexId, LevelOps, SingleId, SpatialIdSet, SpatialIdTable};

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
fn level_f_fills_absolute_range() {
    // f=0 を [0,3] に揃える → 0..=3 が埋まる。
    let table = table_with(25, 0, 100, 100);
    let result = table.level_f(25, 0, 3).unwrap();

    for f in 0..=3 {
        assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
    }
    assert!(!present(&result, 25, 4, 100, 100));
    assert!(!present(&result, 25, -1, 100, 100));
}

#[test]
fn level_f_truncates_overflow() {
    // f=0 と f=20 のセルを [0,5] に揃える → 0..=5 のみ。20 は削られる。
    let mut table = table_with(25, 0, 100, 100);
    table.insert(SingleId::new(25, 20, 100, 100).unwrap(), true);
    let result = table.level_f(25, 0, 5).unwrap();

    for f in 0..=5 {
        assert!(present(&result, 25, f, 100, 100), "f={f} should be filled");
    }
    assert!(!present(&result, 25, 6, 100, 100));
    assert!(!present(&result, 25, 20, 100, 100));
}

#[test]
fn level_f_flattens_relief_across_columns() {
    // 凹凸データ: 列Aは f=0、列Bは f=5（出っ張り）。両方を [0,2] に揃えると
    // どちらの列も同じ f-範囲になり、起伏が平坦化される。
    let mut table = table_with(25, 0, 100, 100); // 列A
    table.insert(SingleId::new(25, 5, 101, 100).unwrap(), true); // 列B（高い）
    let result = table.level_f(25, 0, 2).unwrap();

    for f in 0..=2 {
        assert!(present(&result, 25, f, 100, 100), "A f={f}");
        assert!(present(&result, 25, f, 101, 100), "B f={f}");
    }
    // 元の出っ張り（B の f=5）は消える。
    assert!(!present(&result, 25, 5, 101, 100));
}

#[test]
fn level_f_order_independent() {
    // lo/hi を入れ替えても同じ結果。
    let table = table_with(25, 10, 100, 100);
    let asc = table.level_f(25, 2, 6).unwrap();
    let desc = table.level_f(25, 6, 2).unwrap();

    for f in 2..=6 {
        assert!(present(&asc, 25, f, 100, 100));
        assert!(present(&desc, 25, f, 100, 100));
    }
    assert!(!present(&asc, 25, 10, 100, 100));
}

#[test]
fn level_f_out_of_range_is_error() {
    let table = table_with(25, 0, 100, 100);
    assert!(table.level_f(25, 0, crate::F_MAX[25] + 1).is_err());
    assert!(table.level_f(25, crate::F_MIN[25] - 1, 0).is_err());
}

// ---- X方向（巡回） ----

#[test]
fn level_x_fills_range() {
    let table = table_with(25, 0, 50, 100);
    let result = table.level_x(25, 100, 102).unwrap();

    for x in 100..=102 {
        assert!(present(&result, 25, 0, x, 100), "x={x} should be filled");
    }
    assert!(!present(&result, 25, 0, 50, 100)); // 元の x=50 は捨てられる
    assert!(!present(&result, 25, 0, 103, 100));
}

#[test]
fn level_x_wraps_across_seam() {
    // z=2 は一周4セル。from=3 → to=0 は境界をまたいで x=3 と x=0 を埋める。
    let table = table_with(2, 0, 1, 0);
    let result = table.level_x(2, 3, 0).unwrap();

    assert!(present(&result, 2, 0, 3, 0));
    assert!(present(&result, 2, 0, 0, 0));
    assert!(!present(&result, 2, 0, 1, 0));
    assert!(!present(&result, 2, 0, 2, 0));
}

#[test]
fn level_x_out_of_range_is_error() {
    let table = table_with(2, 0, 0, 0);
    assert!(table.level_x(2, 0, crate::XY_MAX[2] + 1).is_err());
}

// ---- Y方向（境界） ----

#[test]
fn level_y_fills_range() {
    let table = table_with(25, 0, 100, 50);
    let result = table.level_y(25, 100, 103).unwrap();

    for y in 100..=103 {
        assert!(present(&result, 25, 0, 100, y), "y={y} should be filled");
    }
    assert!(!present(&result, 25, 0, 100, 50)); // 元の y=50 は捨てられる
    assert!(!present(&result, 25, 0, 100, 104));
}

#[test]
fn level_y_out_of_range_is_error() {
    let table = table_with(25, 0, 100, 0);
    assert!(table.level_y(25, 0, crate::XY_MAX[25] + 1).is_err());
}

// ---- 総称化の確認 ----

#[test]
fn level_works_on_set() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(25, 9, 100, 100).unwrap());

    let result = set.level_f(25, 0, 2).unwrap();

    for f in 0..=2 {
        let cell = SingleId::new(25, f, 100, 100).unwrap();
        assert!(result.get(&cell).next().is_some(), "f={f} should be filled");
    }
    assert!(
        result
            .get(&SingleId::new(25, 9, 100, 100).unwrap())
            .next()
            .is_none()
    );
}

// 同じ列の複数セルが同じ範囲へ重なるときの衝突解決（値付き Table）。
#[test]
fn level_resolves_overlap_by_policy() {
    // 同一列 (x=100,y=100) の f=0 に値1、f=2 に値9。[0,3] に揃えると全 f で重なる。
    let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
    table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

    let value_at = |t: &SpatialIdTable<u8>, f: i32| {
        let cell = SingleId::new(25, f, 100, 100).unwrap();
        t.get(&cell).next().map(|(_, v)| *v)
    };

    // Max: 重なったセルは max(1, 9) = 9。
    let by_max = table.level_f_with(25, 0, 3, ConflictPolicy::Max).unwrap();
    assert_eq!(value_at(&by_max, 1), Some(9));

    // Min: 重なったセルは min(1, 9) = 1。
    let by_min = table.level_f_with(25, 0, 3, ConflictPolicy::Min).unwrap();
    assert_eq!(value_at(&by_min, 1), Some(1));
}

// FlexId を直接使う混在ズームの確認（Y だけ粗いセルを範囲へ揃える）。
#[test]
fn level_y_on_coarse_cell() {
    let mut table = SpatialIdTable::new();
    table.insert(FlexId::new(2, 0, 2, 2, 1, 1).unwrap(), true); // y は z1 / index1（= z2 の 2,3）
    let result = table.level_y(2, 0, 1).unwrap();

    // y は [0,1] へ揃えられ、元の 2,3 は消える。
    for y in 0..=1 {
        assert!(present(&result, 2, 0, 2, y), "y={y} should be filled");
    }
    assert!(!present(&result, 2, 0, 2, 2));
    assert!(!present(&result, 2, 0, 2, 3));
}
