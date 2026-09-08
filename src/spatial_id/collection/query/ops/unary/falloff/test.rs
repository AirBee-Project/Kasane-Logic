use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::merge_policy::{Max, Sum};
use crate::spatial_id::collection::query::ops::unary::falloff::FalloffPattern;
use crate::{FlexId, SingleId, Source, SpatialIdTable};

/// z=20, f=0, y=0 に固定した行から `x -> value` の対応を取り出す。
fn row(table: &SpatialIdTable<i32>) -> BTreeMap<u32, i32> {
    table
        .flat_single_ids()
        .map(|(sid, v)| (sid.x(), *v))
        .collect()
}

fn time_segment(x: u32, v: i32) -> (SingleId, i32) {
    (SingleId::new(20, 0, x, 0).unwrap(), v)
}

/// 単一Segmentの X falloff（半径2）。減衰は value*(r-|d|)/r。重なりが無いので Sum/Max は同値。
#[test]
fn falloff_x_single_segment() {
    let mut table = SpatialIdTable::new();
    let (id, v) = time_segment(100, 4);
    table.insert(id, v);

    let out = table
        .query()
        .falloff_x(20, 2, None, FalloffPattern::Linear, Sum)
        .collect_table()
        .unwrap();
    let r = row(&out);

    // d=0:4, d=±1:2, d=±2:0
    assert_eq!(r.get(&98), Some(&0));
    assert_eq!(r.get(&99), Some(&2));
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&2));
    assert_eq!(r.get(&102), Some(&0));
}

/// 重なる2Segmentの X falloff を Sum で合成。重なったSegmentは両寄与の和になる。
#[test]
fn falloff_x_overlap_sum() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 4).0, 4);
    table.insert(time_segment(102, 4).0, 4);

    let out = table
        .query()
        .falloff_x(20, 2, None, FalloffPattern::Linear, Sum)
        .collect_table()
        .unwrap();
    let r = row(&out);

    // A(x100) → x98:0 x99:2 x100:4 x101:2 x102:0
    // B(x102) → x100:0 x101:2 x102:4 x103:2 x104:0
    // Sum     → x98:0 x99:2 x100:4 x101:4 x102:4 x103:2 x104:0
    assert_eq!(r.get(&98), Some(&0));
    assert_eq!(r.get(&99), Some(&2));
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&4)); // 2+2
    assert_eq!(r.get(&102), Some(&4)); // 0+4
    assert_eq!(r.get(&103), Some(&2));
    assert_eq!(r.get(&104), Some(&0));
}

/// 同じ入力を Max で合成。重なりは最大値を取るため x101 が Sum と食い違う。
#[test]
fn falloff_x_overlap_max() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 4).0, 4);
    table.insert(time_segment(102, 4).0, 4);

    let out = table
        .query()
        .falloff_x(20, 2, None, FalloffPattern::Linear, Max)
        .collect_table()
        .unwrap();
    let r = row(&out);

    // Max → x100:4 x101:2 x102:4（x101 は max(2,2)=2、Sum の 4 と異なる）
    assert_eq!(r.get(&100), Some(&4));
    assert_eq!(r.get(&101), Some(&2));
    assert_eq!(r.get(&102), Some(&4));
}

/// `lazy_get`で対象範囲だけに絞っても、範囲外にある元Segmentからの減衰寄与を
/// 正しく拾えること（`inverse_bounds`が対象領域を半径分だけ広げてSourceへ問い合わせる）。
#[test]
fn falloff_x_lazy_get_reaches_outside_source() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 4).0, 4); // 元Segmentはx=100のみ

    let query = table
        .query()
        .falloff_x(20, 2, None, FalloffPattern::Linear, Sum);

    // 対象はx=99だけ（元のx=100はこの対象範囲の外）。それでも半径2の減衰で
    // x=99へ2が届くはず。
    let got: Vec<i32> = query
        .lazy_get(time_segment(99, 0).0)
        .unwrap()
        .map(|(_, v)| v)
        .collect();
    assert_eq!(got, alloc::vec![2]);

    // 半径の外（x=200）には何も届かない。
    assert!(
        query
            .lazy_get(time_segment(200, 0).0)
            .unwrap()
            .next()
            .is_none()
    );
}

/// `run_by_segments`(Sourceが持つSegmentごとに`forward_bounds`で影響範囲を求めて評価)が
/// `run()`(全域を一度に評価)と完全に同じ結果を返すこと。Segment同士の影響範囲が重なっても
/// 欠落・重複が起きないことを確認する。
#[test]
fn falloff_x_run_by_segments_matches_run() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 4).0, 4);
    table.insert(time_segment(102, 4).0, 4); // 隣接Segment。影響範囲が重なる。
    table.insert(time_segment(300, 6).0, 6);

    let query = table
        .query()
        .falloff_x(20, 2, None, FalloffPattern::Linear, Sum);

    let mut full: Vec<(FlexId, i32)> = query.run().unwrap().collect();
    full.sort();

    let mut by_segments: Vec<(FlexId, i32)> = query
        .run_by_segments(CancellationToken::never())
        .map(|r| r.unwrap())
        .collect();
    by_segments.sort();

    assert_eq!(full, by_segments);
    assert!(!full.is_empty());
}

/// shift → falloff のように異なる演算を連結しても、`forward_bounds`の合成が正しく
/// 効いて`run()`と一致すること。
#[test]
fn shift_then_falloff_run_by_segments_matches_run() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 4).0, 4);
    table.insert(time_segment(300, 6).0, 6);

    let query = table
        .query()
        .shift_x(20, 50)
        .falloff_x(20, 2, None, FalloffPattern::Linear, Sum);

    let mut full: Vec<(FlexId, i32)> = query.run().unwrap().collect();
    full.sort();

    let mut by_segments: Vec<(FlexId, i32)> = query
        .run_by_segments(CancellationToken::never())
        .map(|r| r.unwrap())
        .collect();
    by_segments.sort();

    assert_eq!(full, by_segments);
    assert!(!full.is_empty());
}

/// 半径0の falloff は恒等（no-op）。
#[test]
fn falloff_x_radius_zero_is_noop() {
    let mut table = SpatialIdTable::new();
    table.insert(time_segment(100, 7).0, 7);

    let out = table
        .query()
        .falloff_x(20, 0, None, FalloffPattern::Linear, Sum)
        .collect_table()
        .unwrap();
    let r = row(&out);

    assert_eq!(r.len(), 1);
    assert_eq!(r.get(&100), Some(&7));
}
