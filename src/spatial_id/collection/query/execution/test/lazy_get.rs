#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, Source, SpatialIdTable};

/// `lazy_get`（`run_on_subset` 経由）は、明示的に `.optimize()` を呼んでいない
/// 素の `Query::Unary` に対しても、内部で `plan_order` によりAST最適化と同じ並び順を
/// 適用したうえで実行される。
///
/// わざと拡大率降順（悪い順序）で `falloff_linear_y(r=3) → falloff_linear_x(r=1)` と書き、
/// `optimize()` を一切呼ばない `lazy_get` の結果が、無最適化の `raw_run_table` 基準値と
/// 一致することを確認する（`.optimize()` を明示的に呼んだ場合も同様に一致することも
/// あわせて確認する）。
#[test]
fn lazy_get_applies_ast_optimization_without_explicit_optimize_call() {
    use crate::spatial_id::collection::query::merge_policy::Max;

    let mut table = SpatialIdTable::<u32>::new();
    table.insert(FlexId::new(10, 10, 10, 100, 10, 100).unwrap(), 50);

    let build = || {
        table
            .clone()
            .query()
            .falloff_linear_y(10, 3, Max)
            .falloff_linear_x(10, 1, Max)
    };

    let expected: SpatialIdTable<u32> = build().raw_run_table().unwrap();
    let target = crate::RangeId::new(10, [10, 10], [90, 110], [90, 110]).unwrap();
    let expected_map: alloc::collections::BTreeMap<_, _> = expected
        .iter()
        .filter(|(id, _)| id.intersects_range(&target.clone().into()))
        .map(|(id, v)| (id, *v))
        .collect();

    // `.optimize()` を呼ばない素の Query::Unary のまま lazy_get する。
    let unoptimized_lazy: alloc::collections::BTreeMap<_, _> =
        build().lazy_get(target.clone()).unwrap().collect();
    assert_eq!(
        unoptimized_lazy, expected_map,
        "optimize()を呼んでいないQuery::UnaryでもAST最適化と同じ並び順で実行されるはず"
    );

    // 明示的に `.optimize()` を呼んだ場合（Query::CommutativeGroup経由）も一致するはず。
    let optimized_lazy: alloc::collections::BTreeMap<_, _> = build()
        .optimize()
        .lazy_get(target.clone())
        .unwrap()
        .collect();
    assert_eq!(optimized_lazy, expected_map, "optimize()後のlazy_getが食い違う");
}

#[test]
fn lazy_view_get_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    let flex_id = FlexId::new(10, 10, 10, 10, 10, 10).unwrap();
    table.insert(flex_id, 42);

    // Normal run
    let expected_result: SpatialIdTable<u32> = table
        .clone()
        .query()
        .shift_x(10, 1)
        .shift_y(10, 2)
        .raw_run_table()
        .unwrap();

    let target = FlexId::new(10, 10, 10, 11, 10, 12).unwrap();

    let expected_val = expected_result.get(&target).next().map(|(_, v)| *v);

    // LazyView get
    let query = table.query().shift_x(10, 1).shift_y(10, 2);

    let mut lazy_iter = query.lazy_get(target).unwrap();
    let lazy_val = lazy_iter.next().map(|(_, v)| v);
    assert_eq!(lazy_iter.next(), None);
    assert_eq!(expected_val, lazy_val);
}

#[test]
fn lazy_view_get_with_default() {
    let mut table = SpatialIdTable::new();
    // 1箇所だけ値を入れる
    let id1 = FlexId::new(10, 10, 10, 10, 10, 10).unwrap();
    table.insert(id1, 100);

    let query = table.query();

    // id1 と、隣接する別のID (値がない) を含む RangeId
    let target = crate::RangeId::new(10, [10, 10], [10, 11], [10, 10]).unwrap();

    // get の場合 (値がある場所しか返らない)
    let results: Vec<_> = query.lazy_get(target.clone()).unwrap().collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 100);

    // get_with_default の場合 (値がない場所は default 値で返る)
    let results_with_default: Vec<_> = query
        .lazy_get_with_default(target.clone(), 0)
        .unwrap()
        .collect();
    assert_eq!(results_with_default.len(), 2);

    // ソートして検証
    let mut results_with_default = results_with_default;
    results_with_default.sort_unstable_by_key(|a| a.0);

    // id1 の場所は元の値 100
    assert_eq!(results_with_default[0].0, id1);
    assert_eq!(results_with_default[0].1, 100);

    // もう一つの場所は default の 0
    let expected_id2 = FlexId::new(10, 10, 10, 11, 10, 10).unwrap();
    assert_eq!(results_with_default[1].0, expected_id2);
    assert_eq!(results_with_default[1].1, 0);
}

/// `lazy_get` は時間軸でも絞り込む。
///
/// 木の走査は時間Segmentの2分割境界で枝刈りするが、ターゲットの秒区間とSegment境界は一致するとは
/// 限らないので、はみ出した葉が残りうる。最終フィルタ（[`FlexId::intersects_range`]）が
/// 時間を見ていないと、同じFlexIdの**別時刻の値**がそのまま返ってしまう。
#[cfg(feature = "temporal_id")]
#[test]
fn lazy_get_filters_by_time() {
    use crate::{Interval, SingleId, SpatialIdTable};
    use alloc::collections::BTreeSet;

    let time_segment = |t: u64| {
        SingleId::new(10, 0, 5, 5)
            .unwrap()
            .with_time(Interval::HOUR, t)
            .unwrap()
    };

    let mut table: SpatialIdTable<i32> = SpatialIdTable::new();
    table.insert(time_segment(0), 1); // [0, 3600)
    table.insert(time_segment(10), 2); // [36000, 39600)

    for (target, expected) in [(time_segment(0), 1), (time_segment(10), 2)] {
        let values: BTreeSet<i32> = table
            .clone()
            .query()
            .lazy_get(target)
            .unwrap()
            .map(|(_, v)| v)
            .collect();
        assert_eq!(
            values,
            [expected].into_iter().collect::<BTreeSet<_>>(),
            "他の時刻の値が混ざっている"
        );
    }
}
