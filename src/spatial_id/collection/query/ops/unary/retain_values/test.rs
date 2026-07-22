use alloc::string::{String, ToString};

use crate::{SingleId, Source, SpatialIdTable};

fn cell(x: u32) -> SingleId {
    SingleId::new(20, 0, x, 100).unwrap()
}

/// `(x, 値)` の対応表へ畳み込む。
fn rows<V: Clone + Ord + Send + Sync + 'static>(
    table: &SpatialIdTable<V>,
) -> alloc::vec::Vec<(u32, V)> {
    let mut out: alloc::vec::Vec<(u32, V)> = table
        .iter()
        .flat_map(|(id, v)| id.clone().single_ids().map(move |s| (s.x(), v.clone())))
        .collect();
    out.sort_by_key(|(x, _)| *x);
    out
}

fn int_table() -> SpatialIdTable<i32> {
    let mut t = SpatialIdTable::new();
    t.insert(cell(10), 1);
    t.insert(cell(11), 5);
    t.insert(cell(12), 10);
    t.insert(cell(13), 20);
    t
}

#[test]
fn retain_value_eq_keeps_only_that_value() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .retain_value_eq(10)
        .raw_run_into()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(12, 10)]);
}

#[test]
fn retain_value_in_range_is_inclusive() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .retain_value_in_range(Some(5), Some(10))
        .raw_run_into()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(11, 5), (12, 10)]);
}

#[test]
fn retain_value_in_range_open_bound() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .retain_value_in_range(Some(10), None)
        .raw_run_into()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(12, 10), (13, 20)]);
}

#[test]
fn retain_value_not_in_range_keeps_the_outside() {
    let out: SpatialIdTable<i32> = int_table()
        .query()
        .retain_value_not_in_range(Some(5), Some(10))
        .raw_run_into()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(10, 1), (13, 20)]);
}

/// 比較に必要なのは `Ord` だけなので、文字列でも同じように絞り込める。
#[test]
fn retain_values_works_for_text() {
    let mut t: SpatialIdTable<String> = SpatialIdTable::new();
    t.insert(cell(10), "apple".to_string());
    t.insert(cell(11), "banana".to_string());
    t.insert(cell(12), "cherry".to_string());

    let out: SpatialIdTable<String> = t
        .query()
        .retain_value_in_range(Some("b".to_string()), Some("bz".to_string()))
        .raw_run_into()
        .unwrap();

    assert_eq!(rows(&out), alloc::vec![(11, "banana".to_string())]);
}

/// 下限 > 上限 は実行前の検証で弾かれる。
#[test]
fn invalid_range_is_rejected_by_validate() {
    let result: Result<SpatialIdTable<i32>, _> = int_table()
        .query()
        .retain_value_in_range(Some(100), Some(1))
        .run_into();

    assert!(matches!(
        result,
        Err(crate::Error::InvalidQueryParameter(_))
    ));
}

/// 遅延評価（対象領域限定）でも同じ絞り込み結果になる。
#[test]
fn retain_values_via_lazy_view() {
    let query = int_table().query().retain_value_in_range(Some(5), Some(10));
    let lazy = query.lazy();

    let got: alloc::vec::Vec<i32> = lazy.get(cell(11)).unwrap().map(|(_, v)| v).collect();
    assert_eq!(got, alloc::vec![5]);

    // 範囲外の値だったセルは何も返らない。
    assert!(lazy.get(cell(13)).unwrap().next().is_none());
}

/// 切り分け: 範囲 (RangeId) を対象にした遅延取得で、複数セルが全て返ること。
#[test]
fn lazy_get_over_range_returns_all_cells() {
    use crate::RangeId;

    let mut t: SpatialIdTable<i32> = SpatialIdTable::new();
    for i in 0..4u32 {
        t.insert(SingleId::new(20, 0, 790000 + i, 500000).unwrap(), i as i32);
    }

    let bbox = RangeId::new(20, [0, 0], [790000, 790003], [500000, 500000]).unwrap();

    let query = t.query();
    let lazy = query.lazy();
    let mut got: alloc::vec::Vec<i32> = lazy.get(bbox).unwrap().map(|(_, v)| v).collect();
    got.sort();

    assert_eq!(got, alloc::vec![0, 1, 2, 3]);
}
