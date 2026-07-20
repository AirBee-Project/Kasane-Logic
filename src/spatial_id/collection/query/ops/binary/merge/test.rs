use crate::spatial_id::collection::query::merge_policy::Sum;
use crate::{SingleId, SpatialIdCollection, SpatialIdTable};

fn cell(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

/// 重なるセルは resolve(a, b)、片側にしかないセルは resolve(a, default) / resolve(default, b)、
/// どちらにも無いセルは空のまま。
#[test]
fn merge_resolves_overlap_and_fills_missing_side_with_default() {
    let mut a = SpatialIdTable::new();
    let mut b = SpatialIdTable::new();

    let (only_a, av) = cell(100, 7); // Aのみ
    let (both, av2) = cell(101, 3); // 両方
    let (_, bv2) = cell(101, 4);
    let (only_b, bv) = cell(102, 5); // Bのみ

    a.insert(only_a.clone(), av);
    a.insert(both.clone(), av2);
    b.insert(both.clone(), bv2);
    b.insert(only_b.clone(), bv);

    let out = a.query().merge(b.query(), 0, Sum).raw_run().unwrap();

    assert_eq!(out.get(&only_a).next().unwrap().1, &7); // resolve(7, default=0)
    assert_eq!(out.get(&both).next().unwrap().1, &7); // resolve(3, 4)
    assert_eq!(out.get(&only_b).next().unwrap().1, &5); // resolve(default=0, 5)

    let neither = cell(103, 0).0;
    assert!(out.get(&neither).next().is_none());
}

/// 両方空のテーブル同士をmergeしても何も起きない（noop）。
#[test]
fn merge_of_two_empty_tables_is_noop() {
    let a = SpatialIdTable::<i32>::new();
    let b = SpatialIdTable::<i32>::new();

    let out = a.query().merge(b.query(), 0, Sum).raw_run().unwrap();

    assert_eq!(out.iter().count(), 0);
}
