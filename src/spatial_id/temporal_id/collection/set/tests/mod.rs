use super::TemporalSet;
use crate::{Interval, TemporalId};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use proptest::prelude::*;

/// 集合が覆う秒。
fn secs(set: &TemporalSet) -> BTreeSet<u64> {
    let mut s = BTreeSet::new();
    for (a, b) in set.ranges() {
        s.extend(a..b);
    }
    s
}

/// `(i, t)` のセル列から集合を構築。
fn build(cells: &[(u64, u64)]) -> TemporalSet {
    let mut set = TemporalSet::new();
    for &(i, t) in cells {
        set.insert(&TemporalId::new(i, t).unwrap());
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
    let iv = set.ranges();
    for w in iv.windows(2) {
        assert!(w[0].1 < w[1].0, "not normalized: {iv:?}");
    }
    for &(s, e) in &iv {
        assert!(s < e, "empty interval: {iv:?}");
    }
}

/// 厳格正解: 秒集合に展開して union/intersection/difference/contains を照合。
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
    for set in sample_sets().iter() {
        let mut rebuilt = TemporalSet::new();
        for c in set.into_iter() {
            rebuilt.insert(&c);
        }
        assert_eq!(secs(set), secs(&rebuilt), "cells roundtrip mismatch");
    }
}

// contains_unixtime の二分探索照合。
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

// WHOLE の扱い: cells() は単一 WHOLE、WHOLE − 1時間 は2区間で有界に分解できる。
#[test]
fn whole_handling() {
    let w = TemporalSet::whole();
    assert!(w.is_whole());
    assert_eq!(
        w.clone().into_iter().collect::<Vec<_>>(),
        alloc::vec![TemporalId::WHOLE]
    );

    let hour = TemporalSet::from(&TemporalId::new(3600_u64, 10).unwrap());
    let d = w.difference(&hour);
    assert_eq!(d.ranges().len(), 2, "WHOLE − 1時間 = 前後2区間");
    assert!(!d.contains_unixtime(36000)); // 穴の中
    assert!(d.contains_unixtime(35999)); // 穴の直前
    assert!(d.contains_unixtime(39600)); // 穴の直後

    // 巨大な残余区間も対数個のセルへ正確に分解できる（爆発しない）
    let cells: Vec<_> = d.into_iter().collect();
    assert!(cells.len() < 400, "cells = {}", cells.len());
    let total: u64 = cells
        .iter()
        .map(|c| c.end_unixtime_exclusive() - c.start_unixtime())
        .sum();
    assert_eq!(total, Interval::WHOLE_SECONDS - 3600);
}

#[test]
fn set_ergonomic_apis() {
    let mut s = TemporalSet::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);

    let t1 = TemporalId::new(3600_u64, 0_u64).unwrap();
    s.insert(&t1);
    assert!(!s.is_empty());
    assert_eq!(s.len(), 1);
    assert!(s.0.iter().any(|(x, _)| x == t1));

    let t2 = TemporalId::new(3600_u64, 1_u64).unwrap(); // start=3600, index=1 (so 3600..7200)
    s.insert(&t2);
    assert_eq!(s.len(), 2);

    s.remove(&t1);
    assert_eq!(s.len(), 1);
    assert!(!s.0.iter().any(|(x, _)| x == t1));

    s.clear();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

/// 回帰テスト: `insert()` の末尾追記高速パスは、隣接する同値レンジを結合した際に
/// より粗いセル分解へ変わり得ること（例: Hour×12 + Hour×12 → Day×1）を考慮せず、
/// 結合前の `count_range` を単純加算していたため `len()` が実際のセル数と食い違っていた。
#[test]
fn len_matches_actual_cell_count_after_adjacent_merge() {
    let mut set = TemporalSet::new();
    // 1時間セルを24個、[0,86400) を隙間なく順番に挿入する（= ちょうど1日）。
    for h in 0..24u64 {
        set.insert(&TemporalId::new(3600_u64, h).unwrap());
    }

    let actual_cells = set.get(TemporalId::WHOLE).count();
    let reported_len = set.len();

    assert_eq!(
        reported_len, actual_cells,
        "len() diverged from actual cell decomposition after adjacent-range merge"
    );
    // 実際に Day 1個へ粗く再分解されていることも確認する。
    assert_eq!(reported_len, 1);
    assert_eq!(set.ranges(), alloc::vec![(0, 86400)]);
}

/// 回帰テスト: 同じ被覆(ranges)でも構築経路が違うと `cached_len` が食い違い、
/// 論理的に等しいはずの `TemporalSet` が `PartialEq` で等しくないと判定されていた
/// （`cached_len` が派生 `PartialEq`/`Eq`/`Hash`/`Ord` に混入していたため）。
#[test]
fn equal_coverage_built_differently_should_be_equal() {
    // 経路A: 1回で1日をまとめて挿入
    let mut a = TemporalSet::new();
    a.insert(&TemporalId::new(86400_u64, 0).unwrap());

    // 経路B: 24個のHourセルを個別に挿入して同じ被覆を作る
    let mut b = TemporalSet::new();
    for h in 0..24u64 {
        b.insert(&TemporalId::new(3600_u64, h).unwrap());
    }

    assert_eq!(
        a.ranges(),
        b.ranges(),
        "same coverage should normalize to same ranges"
    );
    assert_eq!(
        a.len(),
        b.len(),
        "len() must not depend on construction path"
    );
    assert_eq!(
        a, b,
        "same coverage (same ranges) must compare equal regardless of construction path"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `len()` は、どんな順序・粒度で `insert()` を積み重ねても、常に
    /// 実際のセル分解数（`get(WHOLE).count()`）と一致し続けなければならない。
    /// 末尾追記の高速パス（隣接同値マージ）が繰り返し発火する状況を広くカバーする。
    #[test]
    fn prop_len_always_matches_actual_cells(
        // [0, 24) 時間分を Hour 単位で敷き詰め、ランダムに間引く（間引いた箇所が隙間になる）。
        keep in prop::collection::vec(any::<bool>(), 24)
    ) {
        let mut set = TemporalSet::new();
        for (h, keep) in keep.into_iter().enumerate() {
            if keep {
                set.insert(&TemporalId::new(3600_u64, h as u64).unwrap());
            }
        }
        let actual = set.get(TemporalId::WHOLE).count();
        prop_assert_eq!(set.len(), actual);
    }

    /// 挿入順序を入れ替えても（末尾追記の高速パスに乗らない順序も含めて）、
    /// 最終的な `len()` は挿入順序に依存せず一致する。
    #[test]
    fn prop_len_independent_of_insertion_order(
        perm_seed in any::<u64>(),
        keep in prop::collection::vec(any::<bool>(), 24)
    ) {
        let mut hours: Vec<u64> = (0..24u64).filter(|&h| keep[h as usize]).collect();
        // seed から簡易 Fisher-Yates で順序をシャッフルする（乱数クレート追加を避ける）。
        let mut seed = perm_seed;
        for i in (1..hours.len()).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (seed >> 33) as usize % (i + 1);
            hours.swap(i, j);
        }

        let mut set = TemporalSet::new();
        for h in hours {
            set.insert(&TemporalId::new(3600_u64, h).unwrap());
        }
        let actual = set.get(TemporalId::WHOLE).count();
        prop_assert_eq!(set.len(), actual);
    }
}
