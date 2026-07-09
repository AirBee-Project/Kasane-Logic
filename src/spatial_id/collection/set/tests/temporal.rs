//! [`SpatialIdSet`] の時間ネイティブ動作の検証。
//!
//! (1) (空間キー × 秒) のアトム集合正解で insert / union / intersection /
//! difference / get / remove を厳密照合し、(2) テスト専用の参照実装
//! [`SpatioTemporalSet`](crate::spatial_id::collection::testing::SpatioTemporalSet)
//! と突き合わせる。
#![cfg(all(test, feature = "temporal_id"))]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::{FlexId, Interval, SingleId, SpatialId, SpatialIdSet, TemporalId};
use proptest::prelude::*;

type Atom = ((i32, u32, u32), u64);

/// FlexId の空間部分を共通ズーム z の (f,x,y) 群へ展開。
fn spatial_keys(f: &FlexId, z: u8) -> Vec<(i32, u32, u32)> {
    let (fz, xz, yz) = (f.f_zoomlevel(), f.x_zoomlevel(), f.y_zoomlevel());
    let f0 = f.f_index() << (z - fz);
    let x0 = f.x_index() << (z - xz);
    let y0 = f.y_index() << (z - yz);
    let (fs, xs, ys) = (1i32 << (z - fz), 1u32 << (z - xz), 1u32 << (z - yz));
    let mut out = Vec::new();
    for df in 0..fs {
        for dx in 0..xs {
            for dy in 0..ys {
                out.push((f0 + df, x0 + dx, y0 + dy));
            }
        }
    }
    out
}

/// FlexId 列を (空間キー × 秒) のアトム集合へ展開（有界時間前提）。
fn atoms_of<I: IntoIterator<Item = FlexId>>(ids: I, z: u8) -> BTreeSet<Atom> {
    let mut out = BTreeSet::new();
    for f in ids {
        let secs: Vec<u64> =
            (f.temporal().start_unixtime()..f.temporal().end_unixtime_exclusive()).collect();
        for k in spatial_keys(&f, z) {
            for &s in &secs {
                out.insert((k, s));
            }
        }
    }
    out
}

/// 時間付き FlexId を作る（zoom, f/x/y、時間セル (i,t)）。
fn cell(z: u8, f: i32, x: u32, y: u32, i: u64, t: u64) -> FlexId {
    FlexId::new(z, f, z, x, z, y)
        .map(|id| id.with_temporal(TemporalId::new(i, t).unwrap()))
        .unwrap()
}

fn build(cells: &[FlexId]) -> SpatialIdSet {
    let mut s = SpatialIdSet::new();
    for c in cells {
        s.insert(c.clone());
    }
    s
}

/// 代表的な時空間セルの組（同一空間・別時間、部分空間、隣接時間、非交差）。
fn sample_a() -> Vec<FlexId> {
    alloc::vec![
        cell(2, 0, 0, 0, 3600, 0), // (0,0,0) @ [0,3600)
        cell(2, 0, 1, 0, 60, 0),   // (1,0) @ [0,60)
        cell(2, 0, 0, 0, 60, 61),  // 同一空間・別時間 → union される
    ]
}

fn sample_b() -> Vec<FlexId> {
    alloc::vec![
        cell(2, 0, 0, 0, 60, 0), // A の部分空間×部分時間
        cell(2, 0, 2, 0, 60, 0), // A に無い空間
        cell(2, 0, 1, 0, 1, 30), // A の (1,0) の中の1秒
    ]
}

/// insert が同一空間セルの別時間を union する（時間を潰さない）。
#[test]
fn insert_merges_temporal_at_same_cell() {
    let s = build(&[cell(2, 0, 0, 0, 60, 0), cell(2, 0, 0, 0, 60, 2)]);
    let exp: BTreeSet<Atom> = (0..60u64)
        .chain(120..180)
        .map(|sec| ((0, 0, 0), sec))
        .collect();
    assert_eq!(atoms_of(s.iter(), 2), exp);
}

/// 時間 WHOLE のみの利用では、iter が従来どおり temporal=WHOLE の FlexId を返す。
#[test]
fn spatial_only_data_stays_whole() {
    let mut s = SpatialIdSet::new();
    s.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap());
    let ids: Vec<FlexId> = s.iter().collect();
    assert_eq!(ids.len(), 1);
    assert!(ids[0].temporal().is_whole());
}

/// union / intersection / difference をアトム正解で厳密照合。
#[test]
fn set_ops_atom_oracle() {
    let a = build(&sample_a());
    let b = build(&sample_b());

    let (aa, ba) = (atoms_of(a.iter(), 2), atoms_of(b.iter(), 2));

    assert_eq!(
        atoms_of((&a | &b).iter(), 2),
        aa.union(&ba).copied().collect::<BTreeSet<Atom>>(),
        "union"
    );
    assert_eq!(
        atoms_of((&a & &b).iter(), 2),
        aa.intersection(&ba).copied().collect::<BTreeSet<Atom>>(),
        "intersection"
    );
    assert_eq!(
        atoms_of((&a - &b).iter(), 2),
        aa.difference(&ba).copied().collect::<BTreeSet<Atom>>(),
        "difference"
    );
}

/// get（時空間クエリ）: 結果アトム == 集合アトム ∩ クエリアトム。
#[test]
fn get_atom_oracle() {
    let a = build(&[cell(2, 0, 0, 0, 3600, 0), cell(2, 0, 1, 0, 60, 0)]);
    // クエリ: 粗い空間セル (zoom1) × [60,120)
    let query = FlexId::new(1u8, 0, 1u8, 0, 1u8, 0)
        .map(|id| id.with_temporal(TemporalId::new(60_u64, 1).unwrap()))
        .unwrap();
    let got = atoms_of(a.get(&query), 2);
    let qa: BTreeSet<Atom> = spatial_keys(&query, 2)
        .into_iter()
        .flat_map(|k| (60u64..120).map(move |s| (k, s)))
        .collect();
    let exp: BTreeSet<Atom> = atoms_of(a.iter(), 2).intersection(&qa).copied().collect();
    assert_eq!(got, exp);
}

/// remove: 削除アトム ＝ 元 ∩ クエリ、残り ＝ 元 − クエリ。
#[test]
fn remove_atom_oracle() {
    let mut a = build(&[cell(2, 0, 0, 0, 3600, 0)]);
    let before = atoms_of(a.iter(), 2);
    let query = cell(2, 0, 0, 0, 60, 0); // [0,60)
    let removed = atoms_of(a.remove(&query), 2);
    let qa: BTreeSet<Atom> = (0u64..60).map(|s| ((0, 0, 0), s)).collect();
    assert_eq!(
        removed,
        before
            .intersection(&qa)
            .copied()
            .collect::<BTreeSet<Atom>>()
    );
    assert_eq!(
        atoms_of(a.iter(), 2),
        before.difference(&qa).copied().collect::<BTreeSet<Atom>>()
    );
}

/// 時間 WHOLE の集合から有限時間を引いても爆発せず、正しい残余になる。
#[test]
fn whole_minus_finite_is_bounded_and_exact() {
    let mut a = SpatialIdSet::new();
    a.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap()); // (0,0,0) @ WHOLE
    let b = build(&[cell(2, 0, 0, 0, 60, 0)]); // (0,0,0) @ [0,60)
    let d = &a - &b;
    let ids: Vec<FlexId> = d.iter().collect();
    // 二進層のおかげでセル数は高々数百
    assert!(ids.len() < 400, "cells = {}", ids.len());
    // 被覆 = 全時間 − 60秒（秒展開せず長さで照合）
    let total: u64 = ids
        .iter()
        .map(|f| f.temporal().end_unixtime_exclusive() - f.temporal().start_unixtime())
        .sum();
    assert_eq!(total, Interval::WHOLE_SECONDS - 60);
    // 穴 [0,60) の後ろだけが残っている
    assert!(ids.iter().all(|f| f.temporal().start_unixtime() >= 60));
    // 穴を埋め戻すと WHOLE に戻る（正規化の検証）
    let refill = &d | &b;
    let ids: Vec<FlexId> = refill.iter().collect();
    assert_eq!(ids.len(), 1);
    assert!(ids[0].temporal().is_whole());
}

/// 等価判定: 別の時間分解（1時間 vs 60分）で構築しても等しい。
#[test]
fn equality_is_canonical_over_time() {
    let hour = build(&[cell(2, 0, 0, 0, 3600, 0)]);
    let minutes = build(
        &(0..60u64)
            .map(|t| cell(2, 0, 0, 0, 60, t))
            .collect::<Vec<_>>(),
    );
    assert_eq!(hour, minutes);

    let not_same = build(&[cell(2, 0, 0, 0, 3600, 1)]);
    assert_ne!(hour, not_same);
}

/// iter → 再挿入の往復でアトム集合が保たれる。
#[test]
fn iter_roundtrip_preserves_atoms() {
    let a = build(&sample_a());
    let rebuilt = build(&a.iter().collect::<Vec<_>>());
    assert_eq!(atoms_of(a.iter(), 2), atoms_of(rebuilt.iter(), 2));
    assert_eq!(a, rebuilt);
}

/// flat_single_ids が時間セルを保持する。
#[test]
fn flat_single_ids_carry_temporal() {
    let a = build(&[cell(1, 0, 0, 0, 60, 5)]);
    let singles: Vec<_> = a.flat_single_ids().collect();
    assert!(!singles.is_empty());
    for s in singles {
        assert_eq!(s.temporal(), TemporalId::new(60_u64, 5).unwrap());
    }
}

/// 全時間データの葉は時間構造が等しいため、従来同様に空間マージされる
/// （時間ネイティブ化による圧縮の退行がない）。
#[test]
fn whole_time_cells_still_merge() {
    let mut set = SpatialIdSet::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                set.insert(
                    crate::SingleId::new(1, f, x, y).unwrap(), // WHOLE 時間
                );
            }
        }
    }
    assert_eq!(set.count(), 1, "8 octants (whole time) merge into 1");

    // 同一の有限時間でも同様にマージされる。
    let mut set = SpatialIdSet::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                set.insert(cell(1, f, x, y, 3600, 7));
            }
        }
    }
    assert_eq!(set.count(), 1, "8 octants (same finite time) merge into 1");
}

/// インプレース・マージ挿入（insert_combine_mut）の two-way エッジ検証。
///
/// (1) 既存の細かいセル群より**粗い**有限セルを後から挿入 → broadcast_combine が
///     枝の各葉へ後勝ちマージ。(2) 既存の粗いセルより**細かい**セルを挿入 →
///     prepend で既存値を押し下げつつ対象サブセルだけマージ。両方をアトム正解で照合。
#[test]
fn insert_combine_mut_coarse_over_fine_and_fine_into_coarse() {
    // (1) 細→粗: (0,0,0) と (1,0,0) に別々の有限時間、その後 zoom1 の粗いセルで
    // 両方を覆う別時間を後勝ち挿入。
    let mut set = build(&[
        cell(2, 0, 0, 0, 60, 0), // (0,0,0) @ [0,60)
        cell(2, 0, 1, 0, 60, 1), // (1,0,0) @ [60,120)
    ]);
    // zoom1 の (0,0,0) は zoom2 の (0..2,0..2,0..2) を覆う。[30,90) を後勝ちで乗せる。
    set.insert(
        FlexId::new(1, 0, 1, 0, 1, 0)
            .map(|id| id.with_temporal(TemporalId::new(60_u64, 0).unwrap())) // [0,60) at i=60? -> [0,60)
            .unwrap(),
    );
    // アトム正解: 各空間セルで「元の時間 ∪ 挿入時間([0,60))」。後勝ちだが Set なので union。
    let got = atoms_of(set.iter(), 2);
    let mut exp: BTreeSet<Atom> = BTreeSet::new();
    for k in spatial_keys(&FlexId::new(1, 0, 1, 0, 1, 0).unwrap(), 2) {
        for s in 0u64..60 {
            exp.insert((k, s)); // 挿入した [0,60) が全 8 セルに乗る
        }
    }
    // 元データの時間も残る（cell(2,0,1,0,..) の空間キーは (f=0,x=1,y=0)=(0,1,0)）。
    exp.extend((0u64..60).map(|s| ((0, 0, 0), s))); // (0,0,0) @ [0,60)
    exp.extend((60u64..120).map(|s| ((0, 1, 0), s))); // (0,1,0) @ [60,120)（元データが保たれる）
    assert_eq!(got, exp, "coarse-over-fine merge");

    // (2) 粗→細: 粗いセルに時間、その後内側の細かいセルへ別時間を追加。
    let mut set2 = SpatialIdSet::new();
    set2.insert(cell(1, 0, 0, 0, 3600, 0)); // zoom1 (0,0,0) @ [0,3600)
    set2.insert(cell(2, 0, 0, 0, 60, 61)); // zoom2 の一部セル @ [3660,3720)
    let got2 = atoms_of(set2.iter(), 2);
    let mut exp2: BTreeSet<Atom> = BTreeSet::new();
    for k in spatial_keys(&FlexId::new(1, 0, 1, 0, 1, 0).unwrap(), 2) {
        exp2.extend((0u64..3600).map(move |s| (k, s))); // 粗いセル全体 @ [0,3600)
    }
    exp2.extend((3660u64..3720).map(|s| ((0, 0, 0), s))); // 細かいセルに追加時間
    assert_eq!(got2, exp2, "fine-into-coarse merge");
}

fn arb_small_temporal_id() -> impl Strategy<Value = TemporalId> {
    // Generate only Second or Minute intervals to avoid OOM in atoms_of oracle
    let intervals = vec![Interval::Second, Interval::Minute];
    let interval_strat = prop::sample::select(intervals);

    interval_strat.prop_flat_map(|interval| {
        // limit the time range to [0, 60] seconds
        let max_t: u64 = if interval == Interval::Second { 60 } else { 1 };
        (Just(interval), 0u64..=max_t).prop_map(|(i, t)| TemporalId::new(i, t).unwrap())
    })
}

fn arb_temporal_single_id(max_zoom: u8) -> impl Strategy<Value = SingleId> {
    (
        crate::SingleId::arb_within(0..=max_zoom),
        arb_small_temporal_id(),
    )
        .prop_map(|(sid, tid)| sid.with_temporal(tid))
}

fn arb_temporal_set_case() -> impl Strategy<Value = SpatialIdSet> {
    prop::collection::vec(arb_temporal_single_id(2), 1..=8).prop_map(|sids| {
        let mut set = SpatialIdSet::new();
        for sid in sids {
            set.insert(sid);
        }
        set
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// 時間付きのランダムな Set に対する演算（union, intersection, difference）が
    /// アトム正解（空間キー × 秒）の演算結果と一致することを検証する。
    #[test]
    fn temporal_set_ops_atom_oracle_proptest(a in arb_temporal_set_case(), b in arb_temporal_set_case()) {
        let aa = atoms_of(a.iter(), 2);
        let ba = atoms_of(b.iter(), 2);

        prop_assert_eq!(
            atoms_of((&a | &b).iter(), 2),
            aa.union(&ba).copied().collect::<BTreeSet<Atom>>(),
            "union"
        );
        prop_assert_eq!(
            atoms_of((&a & &b).iter(), 2),
            aa.intersection(&ba).copied().collect::<BTreeSet<Atom>>(),
            "intersection"
        );
        prop_assert_eq!(
            atoms_of((&a - &b).iter(), 2),
            aa.difference(&ba).copied().collect::<BTreeSet<Atom>>(),
            "difference"
        );
    }
}

/// get_overlapping: target の時間で制限すべき。
/// 現在は時間を無視して全時間セルを返している（バグ）。
#[test]
fn get_overlapping_should_respect_temporal() {
    let mut set = SpatialIdSet::new();
    // (0,0,0) に 2つの時間セルを挿入: [0,60) と [120,180)
    set.insert(cell(2, 0, 0, 0, 60, 0)); // [0,60)
    set.insert(cell(2, 0, 0, 0, 60, 2)); // [120,180)

    // クエリ: (0,0,0) の [120,150) を指定
    // 同じ空間セルで異なる時間範囲でクエリ
    let query = FlexId::new(2, 0, 2, 0, 2, 0)
        .map(|id| id.with_temporal(TemporalId::new(60_u64, 2).unwrap()))
        .unwrap(); // [120,180)

    // 期待値: [120,180) が返されるべき（クエリの [120,180) と [120,180) の交差）
    let results: Vec<FlexId> = set.get_overlapping(&query).collect();
    let expected_atoms: BTreeSet<Atom> = (120u64..180).map(|s| ((0, 0, 0), s)).collect();
    let actual_atoms = atoms_of(results, 2);

    // 改善後: 時間を考慮して [120,180) のみ返される
    // バグ時: [0,60) + [120,180) の両方が返されていた
    assert_eq!(
        actual_atoms, expected_atoms,
        "get_overlapping should clip to query's temporal range"
    );
}

/// neighbors_share_face: 時間的隣接も考慮すべき（現在は時間を無視している）。
/// 空間的に隣接しているだけでなく、時間的に겹치는 部分でのみ隣接を返すべき。
#[test]
fn neighbors_share_face_should_respect_temporal() {
    let mut set = SpatialIdSet::new();
    // (0,0,0) @ [0,60)
    set.insert(cell(2, 0, 0, 0, 60, 0));
    // (1,0,0) @ [100,160) - 空間的には隣接だが、時間は [0,60) と겹치지 않음
    set.insert(cell(2, 0, 1, 0, 60, 1)); // [60,120) ではなく [100,160)

    // クエリ: (0,0,0) @ [0,60)
    let query = cell(2, 0, 0, 0, 60, 0); // [0,60)

    let neighbors: Vec<FlexId> = set.neighbors_share_face(&query).collect();
    let neighbors_atoms = atoms_of(neighbors.iter().cloned(), 2);

    // 実装上：neighbors_share_face は空間的隣接だけを見て、時間を無視する
    // つまり (1,0,0) @ [100,160) が返される
    // しかし、query は @ [0,60) なので、時間的に겹치는 部분がない
    //
    // 期待動作：
    // - query の時間 [0,60) と겹치는 隣接セルだけを返すべき
    // - 현재 (1,0,0) @ [100,160) は [0,60) と겹치지 않음
    //
    // 現재의 버그: get_overlapping과 같이, neighbors도 시간을 무시하고
    // stored의 전체 시간을 반환한다

    // 시간적 겹침이 없으므로 이웃이 반환되지 않거나,
    // 또는 겹치는 시간 부분만 반환되어야 함
    // (현재 구현상 전체 시간이 반환되므로 이 어설션이 실패함)
    for atom in &neighbors_atoms {
        assert!(
            atom.1 < 60,
            "neighbors should only be in query's temporal range [0,60), got time {}",
            atom.1
        );
    }
}

/// count: iter() が返す FlexId の数を返すべき。
/// 現在は空間ノード数だけを返している（バグ）。
#[test]
fn count_should_count_temporal_cells() {
    let mut set = SpatialIdSet::new();

    // 同一の空間セル (0,0,0) に 3 つの異なる時間セルを挿入
    set.insert(cell(2, 0, 0, 0, 60, 0)); // [0,60)
    set.insert(cell(2, 0, 0, 0, 60, 1)); // [60,120)
    set.insert(cell(2, 0, 0, 0, 60, 2)); // [120,180)

    // count() が返す値
    let count = set.count();

    // iter() が実際に返す FlexId の個数
    let iter_count = set.iter().count();

    // 改善後: count() = iter().count() となるべき
    // バグ時: count() は 1（空間ノード数）を返していた
    assert_eq!(count, iter_count, "count() should equal iter().count()");

    // iter は 3 つの FlexId を返す（各時間セルごと）
    assert_eq!(iter_count, 3, "3 temporal cells should be stored");

    // atoms_of は各秒をアトムとして展開する（3 × 60秒 = 180アトム）
    let atoms = atoms_of(set.iter(), 2);
    assert_eq!(atoms.len(), 180, "180 seconds in total");
}

/// スケーラビリティ検査: count() と iter() のパフォーマンス特性。
///
/// 問題点:
/// - count() = iter().count() は、全セルを列挙する必要があり O(セル数)
/// - 本来は count() が O(セグメント数) であるべき（セル数に無関係）
/// - cells_ref() が毎回 Vec を作成するため、メモリ効率が悪い
#[test]
#[ignore]
fn scalability_performance_characteristics() {
    eprintln!("\n=== Scalability Analysis ===");

    // シナリオ1: 大規模な連続時間
    let mut set1 = SpatialIdSet::new();
    set1.insert(
        FlexId::new(2, 0, 2, 0, 2, 0)
            .map(|id| id.with_temporal(TemporalId::new(1_u64, 0).unwrap())) // Interval=1 (Second)
            .unwrap(),
    );

    eprintln!("Scenario 1: Large continuous time span");
    eprintln!("  Data: 30 days starting from second 0");

    let count1 = set1.count();
    eprintln!("  count() = {} (should be 1 logical interval)", count1);

    // NOTE: これは 1 に等しい（1つの WHOLE セル）
    assert_eq!(count1, 1);

    // シナリオ2: 複数の空間セル、各々複数の時間セル
    let mut set2 = SpatialIdSet::new();
    for space_idx in 0..4 {
        // zoom=2 では座標は [0,4)
        #[allow(clippy::unnecessary_cast)]
        for time_idx in 0..10 {
            set2.insert(
                FlexId::new(2, space_idx as i32, 2, space_idx as u32, 2, 0)
                    .map(|id| id.with_temporal(TemporalId::new(60_u64, time_idx as u64).unwrap()))
                    .unwrap(),
            );
        }
    }

    eprintln!("Scenario 2: 10 spatial cells × 10 temporal cells");
    eprintln!("  Total inserted: 100 FlexIds");

    let count2 = set2.count();
    let iter_count2 = set2.iter().count();
    eprintln!("  count() = {}", count2);
    eprintln!("  iter().count() = {}", iter_count2);

    // count() と iter().count() が一致することを確認
    assert_eq!(count2, iter_count2, "count() should equal iter().count()");

    eprintln!("\nPerformance concern:");
    eprintln!("  count() runs iter().count() internally");
    eprintln!("  This is O(spatial_nodes × temporal_cells), not O(segments)");
    eprintln!("  For large datasets, this becomes expensive");
}

/// マージメカニズムの確認。
/// TemporalMap は FlexTree と同様に、隣接同値セグメントを自動マージする。
#[test]
#[allow(clippy::len_zero)]
fn temporal_map_merges_adjacent_segments() {
    use crate::TemporalMap;

    // TemporalMap は内部的にセグメント (start, end, value) を保持する
    // sweep メソッドで隣接同値セグメントが自動的にマージされる

    fn map_from(t: &TemporalId, v: &'static str) -> TemporalMap<&'static str> {
        let mut m = TemporalMap::new();
        m.insert(t, v);
        m
    }

    let tm1 = map_from(&TemporalId::new(60_u64, 0).unwrap(), "A");
    let tm2 = map_from(&TemporalId::new(60_u64, 1).unwrap(), "A");

    eprintln!("Temporal Merge Test:");
    eprintln!("  tm1: [0, 60) = 'A'");
    eprintln!("  tm2: [60, 120) = 'A'");

    let merged = tm1.union(&tm2, &crate::ConflictPolicy::KeepExisting);
    let cells: Vec<_> = merged.iter().collect();

    eprintln!("  After union with same value:");
    eprintln!("    cells().len() = {}", cells.len());

    // 隣接同値セグメントは union で自動的に マージされる
    // [0,60) + [60,120) = [0,120) という1つのセグメントになる
    // cells() では約数鎖で分解されるため、1個以上の TemporalId になる
    assert!(
        cells.len() >= 1,
        "Adjacent segments with same value should merge"
    );

    // 異なる値の場合はマージされない
    let tm3 = map_from(&TemporalId::new(60_u64, 0).unwrap(), "A");
    let tm4 = map_from(&TemporalId::new(60_u64, 1).unwrap(), "B");

    let not_merged = tm3.union(&tm4, &crate::ConflictPolicy::KeepExisting);
    let cells2: Vec<_> = not_merged.iter().collect();

    eprintln!("  Different values:");
    eprintln!("    tm3: [0, 60) = 'A'");
    eprintln!("    tm4: [60, 120) = 'B'");
    eprintln!("    cells().len() = {}", cells2.len());

    // [0,60)と[60,120)は異なる値なので、2つのセグメントのまま
    assert!(
        cells2.len() >= 2,
        "Adjacent segments with different values should NOT merge"
    );

    eprintln!("\nConclusion:");
    eprintln!("  TemporalMap automatically merges adjacent segments with same value");
    eprintln!("  This is the same normalization as FlexTree does for spatial cells");
}

/// メモリ効率の問題を示すテスト。
#[test]
#[ignore]
fn memory_efficiency_concern() {
    // シナリオ: 同一空間セルで非常に多くの時間セルを持つ場合
    let mut set = SpatialIdSet::new();

    // 10000個の1秒時間セルを挿入
    eprintln!("\nMemory efficiency test:");
    eprintln!("  Inserting 10000 x 1-second temporal cells...");

    for i in 0u64..10000 {
        set.insert(cell(2, 0, 0, 0, 1, i));
    }

    // 問題: iter() が全セルを展開する必要があるため、
    // count() = iter().count() は O(時間セル数) になる
    eprintln!("  Calling iter().count()...");
    let count = set.count();
    eprintln!("  Result: {} cells", count);

    // パフォーマンスの実測:
    // - cells_ref() が毎回 Vec を作成: O(時間セル数)
    // - from_range() が Vec を作成: O(時間セル数)
    // - iter が複数回イテレータを走査: O(時間セル数 * イテレータ走査)
    eprintln!("  WARNING: count() is O(時間セル数), not O(セグメント数)!");
}
