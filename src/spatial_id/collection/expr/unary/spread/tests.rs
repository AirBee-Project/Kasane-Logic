use crate::SpatialIdCollection;
use crate::{ConflictPolicy, SingleId, SpatialIdSet, SpatialIdTable};

fn table_with(z: u8, f: i32, x: u32, y: u32, v: u8) -> SpatialIdTable<u8> {
    let mut table = SpatialIdTable::new();
    table.insert(SingleId::new(z, f, x, y).unwrap(), v);
    table
}

fn value_at(table: &SpatialIdTable<u8>, z: u8, f: i32, x: u32, y: u32) -> Option<u8> {
    let cell = SingleId::new(z, f, x, y).unwrap();
    table.get(&cell).next().map(|(_, v)| *v)
}

/// F 軸沿い（1D）の伝播：高さ方向にだけ広がり、X / Y には広がらない。
#[test]
fn spread_f_propagates_only_along_height() {
    let table = table_with(25, 0, 100, 100, 100);
    let result = table
        .clone()
        .into_query()
        .spread_f(25, 2, |v, dist| {
            let d = v.saturating_sub((dist * 10) as u8);
            (d > 0).then_some(d)
        })
        .run()
        .unwrap();

    // F 方向には ±2 まで減衰しながら広がる。
    assert_eq!(value_at(&result, 25, 0, 100, 100), Some(100));
    assert_eq!(value_at(&result, 25, 1, 100, 100), Some(90));
    assert_eq!(value_at(&result, 25, -2, 100, 100), Some(80));
    assert_eq!(value_at(&result, 25, 3, 100, 100), None);
    // X / Y へは広がらない。
    assert_eq!(value_at(&result, 25, 0, 101, 100), None);
    assert_eq!(value_at(&result, 25, 0, 100, 101), None);
}

/// X 軸沿い（1D）の伝播：X にだけ広がり、Y / F には広がらない。
#[test]
fn spread_x_propagates_only_along_x() {
    let table = table_with(25, 0, 100, 100, 50);
    let result = table
        .clone()
        .into_query()
        .spread_x(25, 1, |v, _| Some(*v))
        .run()
        .unwrap();

    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(50));
    assert_eq!(value_at(&result, 25, 0, 99, 100), Some(50));
    assert_eq!(value_at(&result, 25, 0, 100, 101), None);
    assert_eq!(value_at(&result, 25, 1, 100, 100), None);
}

/// 3D 球（xyz）：F 方向にも広がる。
#[test]
fn spread_xyz_propagates_in_all_axes() {
    let table = table_with(25, 0, 100, 100, 50);
    let result = table
        .clone()
        .into_query()
        .spread_xyz(25, 1, |v, _| Some(*v))
        .run()
        .unwrap();

    // 半径1の球：各軸の隣接が埋まる。
    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(50));
    assert_eq!(value_at(&result, 25, 0, 100, 101), Some(50));
    assert_eq!(value_at(&result, 25, 1, 100, 100), Some(50));
    assert_eq!(value_at(&result, 25, -1, 100, 100), Some(50));
}

/// 軸別の `_with` で衝突方針を指定できる（spread_f_with の Min）。
#[test]
fn spread_f_with_resolves_overlap_by_policy() {
    // F=0 に値1、F=2 に値9。半径1で広げると F=1 で重なる。
    let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
    table.insert(SingleId::new(25, 2, 100, 100).unwrap(), 9u8);

    let identity = |v: &u8, _d: u32| Some(*v);

    let by_min = table
        .clone()
        .into_query()
        .spread_f_with(25, 1, identity, ConflictPolicy::Min)
        .run()
        .unwrap();
    assert_eq!(value_at(&by_min, 25, 1, 100, 100), Some(1));

    let by_max = table
        .clone()
        .into_query()
        .spread_f_with(25, 1, identity, ConflictPolicy::Max)
        .run()
        .unwrap();
    assert_eq!(value_at(&by_max, 25, 1, 100, 100), Some(9));
}

/// 既定の `spread` は XY 平面のみで、F 方向には広がらない。
#[test]
fn spread_default_is_xy_plane_only() {
    let table = table_with(25, 5, 100, 100, 50);
    let result = table
        .clone()
        .into_query()
        .spread(25, 1, |v, _| Some(*v))
        .run()
        .unwrap();

    assert_eq!(value_at(&result, 25, 5, 101, 100), Some(50));
    assert_eq!(value_at(&result, 25, 4, 100, 100), None);
    assert_eq!(value_at(&result, 25, 6, 100, 100), None);
}

/// 中心からの距離に応じて減衰し、円の内側だけが埋まる。
#[test]
fn spread_fills_disc_with_decay() {
    // 半径2・1セルごとに-10で減衰（z=25 = セル自身のズーム）。
    let table = table_with(25, 0, 100, 100, 100);
    let result = table
        .clone()
        .into_query()
        .spread(25, 2, |v, dist| {
            let d = v.saturating_sub((dist * 10) as u8);
            (d > 0).then_some(d)
        })
        .run()
        .unwrap();

    // 中心は減衰なし。
    assert_eq!(value_at(&result, 25, 0, 100, 100), Some(100));
    // 距離1（隣接）は -10。
    assert_eq!(value_at(&result, 25, 0, 101, 100), Some(90));
    assert_eq!(value_at(&result, 25, 0, 100, 99), Some(90));
    // 距離2は -20。
    assert_eq!(value_at(&result, 25, 0, 102, 100), Some(80));
    // (2,2) はユークリッド距離 2.83 > 2 なので円の外 → 伝播しない。
    assert_eq!(value_at(&result, 25, 0, 102, 102), None);
    // 半径の外（距離3）も伝播しない。
    assert_eq!(value_at(&result, 25, 0, 103, 100), None);
}

/// F（高さ）は移動せず、同じ高さ面の上にだけ広がる。
#[test]
fn spread_keeps_height() {
    let table = table_with(25, 5, 100, 100, 50);
    let result = table
        .clone()
        .into_query()
        .spread(25, 1, |v, _| Some(*v))
        .run()
        .unwrap();

    assert_eq!(value_at(&result, 25, 5, 101, 100), Some(50));
    // 別の高さには漏れない。
    assert_eq!(value_at(&result, 25, 4, 101, 100), None);
    assert_eq!(value_at(&result, 25, 6, 101, 100), None);
}

/// `None` を返すと、そのセルには伝播しない（打ち切り）。
#[test]
fn spread_none_stops_propagation() {
    let table = table_with(25, 0, 100, 100, 5);
    // 距離1以上は None。
    let result = table
        .clone()
        .into_query()
        .spread(25, 3, |v, dist| (dist == 0).then_some(*v))
        .run()
        .unwrap();

    assert_eq!(value_at(&result, 25, 0, 100, 100), Some(5));
    assert_eq!(value_at(&result, 25, 0, 101, 100), None);
}

/// 重なりは ConflictPolicy で解決する（既定は Max）。
#[test]
fn spread_resolves_overlap_by_policy() {
    // 隣り合う2セル（値1と値9）を半径1で広げると、中間セルで重なる。
    let mut table: SpatialIdTable<u8> = SpatialIdTable::new();
    table.insert(SingleId::new(25, 0, 100, 100).unwrap(), 1u8);
    table.insert(SingleId::new(25, 0, 102, 100).unwrap(), 9u8);

    let identity = |v: &u8, _d: u32| Some(*v);

    // Max（既定）: 重なる x=101 は max(1, 9) = 9。
    let by_max = table
        .clone()
        .into_query()
        .spread(25, 1, identity)
        .run()
        .unwrap();
    assert_eq!(value_at(&by_max, 25, 0, 101, 100), Some(9));

    // Min: 重なる x=101 は min(1, 9) = 1。
    let by_min = table
        .clone()
        .into_query()
        .spread_with(25, 1, identity, ConflictPolicy::Min)
        .run()
        .unwrap();
    assert_eq!(value_at(&by_min, 25, 0, 101, 100), Some(1));
}

/// `z` がセルより粗いと、半径はその粗いセル単位で測られる（ステップが広がる）。
#[test]
fn spread_radius_uses_given_zoom() {
    // セルは z=25。z=24 で半径1 → 1ステップ = 2 (= 1 << (25-24)) インデックス分。
    let table = table_with(25, 0, 100, 100, 7);
    let result = table
        .clone()
        .into_query()
        .spread(24, 1, |v, _| Some(*v))
        .run()
        .unwrap();

    // x=102 / x=98（±2）は埋まるが、その間の x=101 は埋まらない。
    assert_eq!(value_at(&result, 25, 0, 102, 100), Some(7));
    assert_eq!(value_at(&result, 25, 0, 98, 100), Some(7));
    assert_eq!(value_at(&result, 25, 0, 101, 100), None);
}

/// `z` がセルより細かい場合はエラーにせず、z のセルへ細分化して伝播する。
#[test]
fn spread_finer_zoom_subdivides() {
    // セルは z=24。z=25（1段細かい）で伝播してもエラーにならず、結果が得られる。
    let table = table_with(24, 0, 100, 100, 7);
    let result = table
        .clone()
        .into_query()
        .spread(25, 1, |v, _| Some(*v))
        .run()
        .unwrap();
    assert!(!result.is_empty());
}

/// `z` が最大ズームを超える場合はエラー。
#[test]
fn spread_zoom_over_max_is_error() {
    let table = table_with(25, 0, 100, 100, 7);
    assert!(
        table
            .clone()
            .into_query()
            .spread(u8::MAX, 1, |v, _| Some(*v))
            .run()
            .is_err()
    );
}

/// 値を持たない集合（SpatialIdSet）でも動く。
#[test]
fn spread_works_on_set() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(25, 0, 100, 100).unwrap());

    let result = set
        .clone()
        .into_query()
        .spread(25, 1, |_, _| Some(()))
        .run()
        .unwrap();

    // 隣接セルまで広がる。
    let neighbor = SingleId::new(25, 0, 101, 100).unwrap();
    assert!(result.get(&neighbor).next().is_some());
    // 円の外（距離2）は広がらない。
    let far = SingleId::new(25, 0, 102, 100).unwrap();
    assert!(result.get(&far).next().is_none());
}
