#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, Source, SpatialIdTable};

/// `shift_x` の逆算が、shift のズームより粗いズームへ周期境界（経度方向の折り返し）を
/// 丸め込む際、以前は丸め込み後に隙間が消えているかどうかの判定を誤り、必要な入力領域を
/// 取りこぼすことがあった（x軸の半分しか入力として要求しない）。その結果、`lazy_get` が
/// `run` と食い違う値を返していた。
#[test]
fn lazy_get_shift_x_wrapped_coarsen_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    // z=2 の x=3（東西の折り返し境界寄り）にデータを置く。
    table.insert(FlexId::new(2, 0, 2, 3, 2, 0).unwrap(), 7);

    let expected: SpatialIdTable<u32> = table.clone().query().shift_x(2, -1).run().unwrap();
    let expected_values: Vec<u32> = expected.flat_single_ids(None).map(|(_, v)| *v).collect();

    // shift のズーム(2)より粗い z=1 の x 全域をターゲットにする。
    let target = crate::RangeId::new(1, [-2, 1], [0, 1], [0, 1]).unwrap();
    let lazy_values: Vec<u32> = table
        .query()
        .shift_x(2, -1)
        .lazy_get(target)
        .unwrap()
        .map(Result::unwrap)
        .map(|(_, v)| v)
        .collect();

    assert_eq!(lazy_values, expected_values);
}

/// x系演算子を2つ連鎖すると、1つ目の演算子の`reverse_target`が返す中間`bounds`が
/// 折り返し状態（`x[0] > x[1]`）になることがある。木の枝刈り
/// （`Node::overlapping_children_range`）がこの規約を考慮していなかったため、
/// `lazy_get`が`run`と食い違う（データを取りこぼす）ことがあった。
#[test]
fn lazy_get_chained_shift_x_wrapped_intermediate_bounds_matches_run() {
    let mut table = SpatialIdTable::<u32>::new();
    table.insert(FlexId::new(2, 0, 2, 2, 2, 0).unwrap(), 9);

    let expected: SpatialIdTable<u32> = table
        .clone()
        .query()
        .shift_x(2, -3)
        .shift_x(2, -2)
        .run()
        .unwrap();
    let expected_values: Vec<u32> = expected.flat_single_ids(None).map(|(_, v)| *v).collect();

    // ターゲットは shift と同じズーム(2)の x 全域。中間 bounds が折り返し表現になる。
    let target = crate::RangeId::new(2, 0, [0, 3], 0).unwrap();
    let lazy_values: Vec<u32> = table
        .query()
        .shift_x(2, -3)
        .shift_x(2, -2)
        .lazy_get(target)
        .unwrap()
        .map(Result::unwrap)
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
        .run()
        .unwrap();

    let target = FlexId::new(10, 10, 10, 11, 10, 12).unwrap();

    let expected_val = expected_result.get(target).next().map(|(_, v)| *v);

    // LazyView get
    let query = table.query().shift_x(10, 1).shift_y(10, 2);

    let mut lazy_iter = query.lazy_get(target).unwrap().map(Result::unwrap);
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
    let results: Vec<_> = query
        .lazy_get(target.clone())
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 100);

    // get_with_default の場合 (値がない場所は default 値で返る)
    let results_with_default: Vec<_> = query
        .lazy_get_with_default(target.clone(), 0)
        .unwrap()
        .map(Result::unwrap)
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
            .map(Result::unwrap)
            .map(|(_, v)| v)
            .collect();
        assert_eq!(
            values,
            [expected].into_iter().collect::<BTreeSet<_>>(),
            "他の時刻の値が混ざっている"
        );
    }
}

#[test]
fn lazy_get_directional_falloff_matches_run() {
    use crate::Side;
    use crate::spatial_id::collection::query::merge_policy::Sum;
    use crate::spatial_id::collection::query::ops::unary::falloff::FalloffPattern;

    let mut table = SpatialIdTable::<u32>::new();
    table.insert(FlexId::new(4, 2, 4, 2, 4, 2).unwrap(), 10);

    for side in [Some(Side::Upper), Some(Side::Lower), None] {
        let q_f = table
            .clone()
            .query()
            .falloff_f(4, 2, side, FalloffPattern::Linear, Sum);
        let expected_f = table
            .clone()
            .query()
            .falloff_f(4, 2, side, FalloffPattern::Linear, Sum)
            .run()
            .unwrap();
        let target_f = FlexId::new(4, 3, 4, 2, 4, 2).unwrap();
        assert_eq!(
            q_f.lazy_get(target_f)
                .unwrap()
                .map(Result::unwrap)
                .next()
                .map(|(_, v)| v),
            expected_f.get(target_f).next().map(|(_, v)| *v),
        );

        let q_x = table
            .clone()
            .query()
            .falloff_x(4, 2, side, FalloffPattern::Linear, Sum);
        let expected_x = table
            .clone()
            .query()
            .falloff_x(4, 2, side, FalloffPattern::Linear, Sum)
            .run()
            .unwrap();
        let target_x = FlexId::new(4, 2, 4, 3, 4, 2).unwrap();
        assert_eq!(
            q_x.lazy_get(target_x)
                .unwrap()
                .map(Result::unwrap)
                .next()
                .map(|(_, v)| v),
            expected_x.get(target_x).next().map(|(_, v)| *v),
        );

        let q_y = table
            .clone()
            .query()
            .falloff_y(4, 2, side, FalloffPattern::Linear, Sum);
        let expected_y = table
            .clone()
            .query()
            .falloff_y(4, 2, side, FalloffPattern::Linear, Sum)
            .run()
            .unwrap();
        let target_y = FlexId::new(4, 2, 4, 2, 4, 1).unwrap();
        assert_eq!(
            q_y.lazy_get(target_y)
                .unwrap()
                .map(Result::unwrap)
                .next()
                .map(|(_, v)| v),
            expected_y.get(target_y).next().map(|(_, v)| *v),
        );
    }
}

#[test]
fn lazy_get_zoom_out_wrapped_target_matches_run() {
    use crate::spatial_id::collection::query::merge_policy::Sum;

    let mut table = SpatialIdTable::<u32>::new();
    table.insert(FlexId::new(3, 0, 3, 0, 3, 0).unwrap(), 11);
    table.insert(FlexId::new(3, 0, 3, 7, 3, 0).unwrap(), 13);

    let expected = table.clone().query().zoom_out(2, Sum).run().unwrap();
    let expected_vals: Vec<u32> = expected.flat_single_ids(None).map(|(_, v)| *v).collect();

    let mut target = crate::RangeId::new(3, 0, 0, 0).unwrap();
    target.set_x([3, 2]).unwrap();

    let lazy_vals: Vec<u32> = table
        .query()
        .zoom_out(2, Sum)
        .lazy_get(target)
        .unwrap()
        .map(Result::unwrap)
        .map(|(_, v)| v)
        .collect();
    assert_eq!(lazy_vals, expected_vals);
}

/// 1つ値を返したあと、読み出しに失敗する入力源。
struct FailingSource;

impl crate::Source for FailingSource {
    type Value = u32;

    fn read_flex_ids<'a>(
        &'a self,
        _targets: &[crate::RangeId],
        _token: &crate::CancellationToken,
    ) -> Result<crate::spatial_id::collection::query::traits::QueryIter<'a, u32>, crate::Error>
    {
        let items = [
            Ok((FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 1)),
            Err(crate::Error::SourceRead("broken".into())),
        ];
        Ok(
            crate::spatial_id::collection::query::traits::QueryIter::from_ordered(
                items.into_iter(),
            ),
        )
    }
}

#[test]
fn lazy_get_yields_source_errors() {
    let target = crate::RangeId::new(1, [-2, 1], [0, 1], [0, 1]).unwrap();
    let items: Vec<_> = FailingSource.query().lazy_get(target).unwrap().collect();
    assert!(matches!(
        items.as_slice(),
        [Ok(_), Err(crate::Error::SourceRead(_))]
    ));
}

#[test]
fn lazy_get_with_default_stops_at_error() {
    let target = crate::RangeId::new(1, [-2, 1], [0, 1], [0, 1]).unwrap();
    let items: Vec<_> = FailingSource
        .query()
        .lazy_get_with_default(target, 0)
        .unwrap()
        .collect();
    // 失敗した場所を既定値で埋めず、エラーで打ち切る
    assert!(matches!(
        items.as_slice(),
        [Ok(_), Err(crate::Error::SourceRead(_))]
    ));
}
