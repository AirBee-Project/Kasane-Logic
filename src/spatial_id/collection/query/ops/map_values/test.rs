use alloc::string::{String, ToString};

use crate::{SingleId, Source, SpatialIdTable};

fn cell(x: u32) -> SingleId {
    SingleId::new(20, 0, x, 200).unwrap()
}

fn int_table() -> SpatialIdTable<i32> {
    let mut t = SpatialIdTable::new();
    t.insert(cell(10), 1);
    t.insert(cell(11), 15);
    t
}

/// 数値 → 真偽値のように、値の**型そのもの**を変えられる。
#[test]
fn map_values_changes_the_value_type() {
    let out: SpatialIdTable<bool> = int_table()
        .query()
        .map_values(|v: i32| v > 10)
        .raw_run()
        .unwrap();

    let mut rows: alloc::vec::Vec<(u32, bool)> = out
        .iter()
        .flat_map(|(id, v)| id.clone().single_ids().map(move |s| (s.x(), *v)))
        .collect();
    rows.sort_by_key(|(x, _)| *x);

    assert_eq!(rows, alloc::vec![(10, false), (11, true)]);
}

/// 数値 → 文字列。変換表のような任意の写像が使える。
#[test]
fn map_values_to_text() {
    let out: SpatialIdTable<String> = int_table()
        .query()
        .map_values(|v: i32| if v > 10 { "high" } else { "low" }.to_string())
        .raw_run()
        .unwrap();

    let mut rows: alloc::vec::Vec<(u32, String)> = out
        .iter()
        .flat_map(|(id, v)| {
            let v = v.clone();
            id.clone().single_ids().map(move |s| (s.x(), v.clone()))
        })
        .collect();
    rows.sort_by_key(|(x, _)| *x);

    assert_eq!(
        rows,
        alloc::vec![(10, "low".to_string()), (11, "high".to_string())]
    );
}

/// 変換の前後で他の演算子と組み合わせられる（変換前に絞り、変換後にも絞る）。
#[test]
fn map_values_composes_with_other_operators() {
    let out: SpatialIdTable<bool> = int_table()
        .query()
        .filter_in(10..) // 変換前: 10以上だけ残す
        .map_values(|v: i32| v > 10) // 型変換
        .filter_eq(true) // 変換後: true だけ残す
        .raw_run()
        .unwrap();

    let rows: alloc::vec::Vec<u32> = out
        .iter()
        .flat_map(|(id, _)| id.clone().single_ids().map(|s| s.x()))
        .collect();

    assert_eq!(rows, alloc::vec![11]);
}

/// 遅延評価でも型変換が効く（対象領域だけを読んで変換する）。
#[test]
fn map_values_via_lazy_view() {
    let query = int_table().query().map_values(|v: i32| v > 10);
    let lazy = query.lazy();

    let got: alloc::vec::Vec<bool> = lazy.get(cell(11)).unwrap().map(|(_, v)| v).collect();
    assert_eq!(got, alloc::vec![true]);

    let got: alloc::vec::Vec<bool> = lazy.get(cell(10)).unwrap().map(|(_, v)| v).collect();
    assert_eq!(got, alloc::vec![false]);
}
