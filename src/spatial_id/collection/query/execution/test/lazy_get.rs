#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, Source, SpatialIdTable};

/// `shift_x` の逆算が、shift のズームより粗いズームへ周期境界（経度方向の折り返し）を
/// 丸め込む際、以前は丸め込み後に隙間が消えているかどうかの判定を誤り、必要な入力領域を
/// 取りこぼすことがあった（x軸の半分しか入力として要求しない）。その結果、`lazy_get` が
/// `raw_run` と食い違う値を返していた。
#[test]
fn lazy_get_shift_x_wrapped_coarsen_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    // z=2 の x=3（東西の折り返し境界寄り）にデータを置く。
    table.insert(FlexId::new(2, 0, 2, 3, 2, 0).unwrap(), 7);

    let expected: SpatialIdTable<u32> = table.clone().query().shift_x(2, -1).raw_run().unwrap();
    let expected_values: Vec<u32> = expected.flat_single_ids().map(|(_, v)| *v).collect();

    // shift のズーム(2)より粗い z=1 の x 全域をターゲットにする。
    let target = crate::RangeId::new(1, [-2, 1], [0, 1], [0, 1]).unwrap();
    let lazy_values: Vec<u32> = table
        .query()
        .shift_x(2, -1)
        .lazy_get(target)
        .unwrap()
        .map(|(_, v)| v)
        .collect();

    assert_eq!(lazy_values, expected_values);
}

/// x系演算子を2つ連鎖すると、1つ目の演算子の`inverse_bounds`が返す中間`bounds`が
/// 折り返し状態（`x[0] > x[1]`）になることがある。木の枝刈り
/// （`Node::overlapping_children_range`）がこの規約を考慮していなかったため、
/// `lazy_get`が`raw_run`と食い違う（データを取りこぼす）ことがあった。
#[test]
fn lazy_get_chained_shift_x_wrapped_intermediate_bounds_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    table.insert(FlexId::new(2, 0, 2, 2, 2, 0).unwrap(), 9);

    let expected: SpatialIdTable<u32> = table
        .clone()
        .query()
        .shift_x(2, -3)
        .shift_x(2, -2)
        .raw_run()
        .unwrap();
    let expected_values: Vec<u32> = expected.flat_single_ids().map(|(_, v)| *v).collect();

    // ターゲットは shift と同じズーム(2)の x 全域。中間 bounds が折り返し表現になる。
    let target = crate::RangeId::new(2, 0, [0, 3], 0).unwrap();
    let lazy_values: Vec<u32> = table
        .query()
        .shift_x(2, -3)
        .shift_x(2, -2)
        .lazy_get(target)
        .unwrap()
        .map(|(_, v)| v)
        .collect();

    assert_eq!(lazy_values, expected_values);
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
        .raw_run()
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
