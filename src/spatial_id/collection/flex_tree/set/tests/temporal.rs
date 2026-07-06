//! [`SpatialIdSet`] の時間ネイティブ動作の検証。
//!
//! (1) (空間キー × 秒) のアトム集合オラクルで insert / union / intersection /
//! difference / get / remove を厳密照合し、(2) テスト専用の参照実装
//! [`SpatioTemporalSet`](crate::spatial_id::collection::spatio_temporal::SpatioTemporalSet)
//! と突き合わせる。
#![cfg(all(test, feature = "temporal_id"))]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::spatial_id::collection::spatio_temporal::SpatioTemporalSet;
use crate::{FlexId, SpatialId, SpatialIdSet, TemporalId};

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
    FlexId::new_with_temporal(z, f, z, x, z, y, TemporalId::from_seconds(i, t).unwrap()).unwrap()
}

fn build(cells: &[FlexId]) -> SpatialIdSet {
    let mut s = SpatialIdSet::new();
    for c in cells {
        s.insert(c.clone());
    }
    s
}

fn build_ref(cells: &[FlexId]) -> SpatioTemporalSet {
    let mut s = SpatioTemporalSet::new();
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

/// union / intersection / difference をアトムオラクルで厳密照合。
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

/// 参照実装（SpatioTemporalSet）との突き合わせ。
#[test]
fn matches_reference_implementation() {
    let (ca, cb) = (sample_a(), sample_b());
    let (a, b) = (build(&ca), build(&cb));
    let (ra, rb) = (build_ref(&ca), build_ref(&cb));

    assert_eq!(atoms_of(a.iter(), 2), atoms_of(ra.iter(), 2), "insert");
    assert_eq!(
        atoms_of((&a | &b).iter(), 2),
        atoms_of(ra.union(&rb).iter(), 2),
        "union"
    );
    assert_eq!(
        atoms_of((&a & &b).iter(), 2),
        atoms_of(ra.intersection(&rb).iter(), 2),
        "intersection"
    );
    assert_eq!(
        atoms_of((&a - &b).iter(), 2),
        atoms_of(ra.difference(&rb).iter(), 2),
        "difference"
    );
}

/// get（時空間クエリ）: 結果アトム == 集合アトム ∩ クエリアトム。
#[test]
fn get_atom_oracle() {
    let a = build(&[cell(2, 0, 0, 0, 3600, 0), cell(2, 0, 1, 0, 60, 0)]);
    // クエリ: 粗い空間セル (zoom1) × [60,120)
    let query = FlexId::new_with_temporal(
        1u8,
        0,
        1u8,
        0,
        1u8,
        0,
        TemporalId::from_seconds(60, 1).unwrap(),
    )
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
    assert_eq!(total, TemporalId::DOMAIN_END - 60);
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
        assert_eq!(*s.temporal(), TemporalId::from_seconds(60, 5).unwrap());
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
