#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, Source, SpatialIdTable};

#[test]
fn lazy_view_get_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    let flex_id = FlexId::new(10, 10, 10, 10, 10, 10).unwrap();
    table.insert(flex_id.clone(), 42);

    // Normal run
    let expected_result: SpatialIdTable<u32> = table
        .clone()
        .query()
        .shift_x(10, 1)
        .shift_y(10, 2)
        .raw_run_into()
        .unwrap();

    let target = FlexId::new(10, 10, 10, 11, 10, 12).unwrap();

    let expected_val = expected_result.get(&target).next().map(|(_, v)| *v);

    // LazyView get
    let query = table.query().shift_x(10, 1).shift_y(10, 2);
    let lazy_view = query.lazy();

    let mut lazy_iter = lazy_view.get(target.clone()).unwrap();
    let lazy_val = lazy_iter.next().map(|(_, v)| v);
    assert_eq!(lazy_iter.next(), None);
    assert_eq!(expected_val, lazy_val);
}

#[test]
fn lazy_view_get_with_default() {
    let mut table = SpatialIdTable::new();
    // 1箇所だけ値を入れる
    let id1 = FlexId::new(10, 10, 10, 10, 10, 10).unwrap();
    table.insert(id1.clone(), 100);

    let query = table.query();
    let lazy_view = query.lazy();

    // id1 と、隣接する別のID (値がない) を含む RangeId
    let target = crate::RangeId::new(10, [10, 10], [10, 11], [10, 10]).unwrap();

    // get の場合 (値がある場所しか返らない)
    let results: Vec<_> = lazy_view.get(target.clone()).unwrap().collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 100);

    // get_with_default の場合 (値がない場所は default 値で返る)
    let results_with_default: Vec<_> = lazy_view
        .get_with_default(target.clone(), 0)
        .unwrap()
        .collect();
    assert_eq!(results_with_default.len(), 2);

    // ソートして検証
    let mut results_with_default = results_with_default;
    results_with_default.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    // id1 の場所は元の値 100
    assert_eq!(results_with_default[0].0, id1);
    assert_eq!(results_with_default[0].1, 100);

    // もう一つの場所は default の 0
    let expected_id2 = FlexId::new(10, 10, 10, 11, 10, 10).unwrap();
    assert_eq!(results_with_default[1].0, expected_id2);
    assert_eq!(results_with_default[1].1, 0);
}
