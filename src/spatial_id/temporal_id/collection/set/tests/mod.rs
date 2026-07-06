use super::TemporalSet;
use crate::{Interval, TemporalId};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// 集合が覆う秒（有界ドメイン前提。WHOLE は使わない）。
fn secs(set: &TemporalSet) -> BTreeSet<u64> {
    let mut s = BTreeSet::new();
    for &(a, b) in set.intervals() {
        s.extend(a..b);
    }
    s
}

/// `(i, t)` のセル列から集合を構築。
fn build(cells: &[(u64, u64)]) -> TemporalSet {
    let mut set = TemporalSet::new();
    for &(i, t) in cells {
        set.insert(&TemporalId::from_seconds(i, t).unwrap());
    }
    set
}

/// [0, 7200) に収まる代表集合（重なり・隣接・入れ子）。
fn sample_sets() -> Vec<TemporalSet> {
    alloc::vec![
        TemporalSet::new(),
        build(&[(60, 0)]),
        build(&[(60, 0), (60, 2)]),
        build(&[(3600, 0)]),
        build(&[(3600, 0), (60, 60)]), // 隣接 → [0,3660)
        build(&[(1, 0), (1, 1), (1, 3)]),
        build(&[(60, 30), (3600, 1)]),
        build(&[(3600, 0), (3600, 1)]), // [0,7200)
    ]
}

/// 正規化不変条件: 昇順・互いに素・隣接非連結。
fn assert_normalized(set: &TemporalSet) {
    for w in set.intervals().windows(2) {
        assert!(w[0].1 < w[1].0, "not normalized: {:?}", set.intervals());
    }
    for &(s, e) in set.intervals() {
        assert!(s < e, "empty interval: {:?}", set.intervals());
    }
}

/// 厳格オラクル: 秒集合に展開して union/intersection/difference/contains を照合。
#[test]
fn set_algebra_oracle() {
    let sets = sample_sets();
    for a in &sets {
        let sa = secs(a);
        for b in &sets {
            let sb = secs(b);

            let u = a.union(b);
            assert_eq!(secs(&u), sa.union(&sb).copied().collect());
            assert_normalized(&u);

            let i = a.intersection(b);
            assert_eq!(secs(&i), sa.intersection(&sb).copied().collect());
            assert_normalized(&i);

            let d = a.difference(b);
            assert_eq!(secs(&d), sa.difference(&sb).copied().collect());
            assert_normalized(&d);
        }
    }
}

/// `cells()` が被覆を保つ（往復: 集合 → セル列 → 集合 で秒集合が一致）。
#[test]
fn cells_roundtrip_preserves_coverage() {
    for set in sample_sets() {
        let mut rebuilt = TemporalSet::new();
        for c in set.cells() {
            rebuilt.insert(&c);
        }
        assert_eq!(secs(&set), secs(&rebuilt), "cells roundtrip mismatch");
    }
}

/// contains の照合（秒の部分集合判定と一致）。
#[test]
fn contains_oracle() {
    let sets = sample_sets();
    let probes = [
        TemporalId::from_seconds(60, 0).unwrap(),
        TemporalId::from_seconds(60, 1).unwrap(),
        TemporalId::from_seconds(3600, 0).unwrap(),
        TemporalId::from_seconds(1, 3600).unwrap(),
    ];
    for set in &sets {
        let s = secs(set);
        for p in &probes {
            let ps: BTreeSet<u64> = (p.start_unixtime()..p.end_unixtime_exclusive()).collect();
            assert_eq!(
                set.contains(p),
                ps.is_subset(&s),
                "contains {set:?} ⊇ {p:?}"
            );
        }
    }
}

/// contains_unixtime の二分探索照合。
#[test]
fn contains_unixtime_oracle() {
    for set in sample_sets() {
        let s = secs(&set);
        for probe in [0u64, 1, 59, 60, 61, 3599, 3600, 3659, 3660, 7199, 7200] {
            assert_eq!(
                set.contains_unixtime(probe),
                s.contains(&probe),
                "contains_unixtime({probe}) mismatch: {set:?}"
            );
        }
    }
}

/// WHOLE の扱い: cells() は単一 WHOLE、WHOLE − 1時間 は2区間で有界に分解できる。
#[test]
fn whole_handling() {
    let w = TemporalSet::whole();
    assert!(w.is_whole());
    assert_eq!(w.cells(), alloc::vec![TemporalId::WHOLE]);

    let hour = TemporalSet::from_temporal(&TemporalId::from_seconds(3600, 10).unwrap());
    let d = w.difference(&hour);
    assert_eq!(d.intervals().len(), 2, "WHOLE − 1時間 = 前後2区間");
    assert!(!d.contains_unixtime(36000)); // 穴の中
    assert!(d.contains_unixtime(35999)); // 穴の直前
    assert!(d.contains_unixtime(39600)); // 穴の直後

    // 巨大な残余区間も対数個のセルへ正確に分解できる（爆発しない）
    let cells = d.cells();
    assert!(cells.len() < 400, "cells = {}", cells.len());
    let total: u64 = cells
        .iter()
        .map(|c| c.end_unixtime_exclusive() - c.start_unixtime())
        .sum();
    assert_eq!(total, Interval::WHOLE_SECONDS - 3600);

    // 窓で限定した分解
    let window = TemporalId::from_seconds(3600, 11).unwrap(); // [39600, 43200)
    let cells = d.cells_clipped(&window);
    assert_eq!(cells, alloc::vec![window]);
}
