#[cfg(test)]
use alloc::vec::Vec;

use crate::spatial_id::collection::query::merge_policy::Sum;
use crate::{FlexId, RangeId, SingleId, Source, SpatialIdTable};

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

/// 回帰テスト: `run_on_subset`（`lazy_get` の実体）は `&self` のままでも
/// [`plan_order`](crate::spatial_id::collection::query::execution::group_commutative::plan_order)
/// で `run()` と同じ並べ替え規則を適用する。
///
/// `falloff_linear_x` → `shift_x` → `shift_y` は互いに可換（Separable、shiftは単射なので
/// ポリシー不一致でも可換）だが、宣言順は `expansion_ratio` 昇順になっていない
/// （falloffが先）。並べ替え（`plan_order`）とその逆変換（`inverse_bounds` の逆順折り畳み）が
/// 一致していないと、遅延経路だけ宣言順そのままの `raw_run` と結果が食い違うはず。
#[test]
fn lazy_get_reorders_commutative_ops_and_still_matches_raw_run() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(12, 0, 100, 100).unwrap(), 7);

    let build = |t: SpatialIdTable<i32>| {
        t.query()
            .falloff_linear_x(12, 2, Sum)
            .shift_x(12, 5)
            .shift_y(12, -3)
    };

    let expected: SpatialIdTable<i32> = build(table.clone()).raw_run_table().unwrap();

    let query = build(table.clone());
    let bbox = RangeId::new(12, [0, 0], [0, 4095], [0, 4095]).unwrap();
    let mut got: Vec<(FlexId, i32)> = query.lazy_get(bbox).unwrap().collect();
    got.sort();

    let mut expected_pairs: Vec<(FlexId, i32)> = expected.iter().map(|(id, &v)| (id, v)).collect();
    expected_pairs.sort();

    assert_eq!(got, expected_pairs);
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
