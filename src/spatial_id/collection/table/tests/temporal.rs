//! [`SpatialIdTable`] / [`SpatialIdMap`] の時間ネイティブ動作の検証。
//!
//! (1) `(空間キー × 秒) → 値` の写像正解で insert（後勝ち）/ get / remove /
//! iter / value_get を厳密照合し、(2) テスト専用の参照実装
//! [`SpatioTemporalTable`](crate::spatial_id::collection::testing::SpatioTemporalTable)
//! と突き合わせる。
#![cfg(all(test, feature = "temporal_id"))]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::{FlexId, Interval, SpatialId, SpatialIdMap, SpatialIdTable, TemporalId};

type Key = ((i32, u32, u32), u64);

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

/// (FlexId, 値) 列を `(空間キー, 秒) → 値` の写像へ展開（有界時間前提）。
fn atom_map<I: IntoIterator<Item = (FlexId, i32)>>(pairs: I, z: u8) -> BTreeMap<Key, i32> {
    let mut out = BTreeMap::new();
    for (f, v) in pairs {
        let secs: Vec<u64> =
            (f.temporal().start_unixtime()..f.temporal().end_unixtime_exclusive()).collect();
        for k in spatial_keys(&f, z) {
            for &s in &secs {
                out.insert((k, s), v);
            }
        }
    }
    out
}

/// 時間付き FlexId を作る
fn make_flex_id(z: u8, f: i32, x: u32, y: u32, i: u64, t: u64) -> FlexId {
    FlexId::new(z, f, z, x, z, y)
        .map(|id| id.with_temporal(TemporalId::new(i, t).unwrap()))
        .unwrap()
}

/// 逐次 insert の期待値（後勝ち）をアトム写像で構築する正解。
fn oracle(entries: &[(FlexId, i32)], z: u8) -> BTreeMap<Key, i32> {
    let mut out = BTreeMap::new();
    for (f, v) in entries {
        for (k, _) in atom_map([(f.clone(), *v)], z) {
            out.insert(k, *v);
        }
    }
    out
}

fn table_of(entries: &[(FlexId, i32)]) -> SpatialIdTable<i32> {
    let mut t = SpatialIdTable::new();
    for (f, v) in entries {
        t.insert(f.clone(), *v);
    }
    t
}

fn map_of(entries: &[(FlexId, i32)]) -> SpatialIdMap<i32> {
    let mut m = SpatialIdMap::new();
    for (f, v) in entries {
        m.insert(f.clone(), *v);
    }
    m
}

/// 代表的な時空間エントリ（同一時空間の上書き、部分時間の上書き、同一空間・別時間、別空間）。
fn sample_entries() -> Vec<(FlexId, i32)> {
    alloc::vec![
        (make_flex_id(2, 0, 0, 0, 3600, 0), 1), // (0,0,0) @ [0,3600) = 1
        (make_flex_id(2, 0, 0, 0, 60, 1), 2),   // 部分時間 [60,120) を 2 で上書き
        (make_flex_id(2, 0, 1, 0, 60, 0), 3),   // 別空間
        (make_flex_id(2, 0, 0, 0, 60, 100), 4), // 同一空間・別時間（時間外挿入）
        (make_flex_id(2, 0, 1, 0, 1, 30), 5),   // (1,0) の中の1秒を上書き
    ]
}

/// Table: insert（後勝ち）のアトム写像が正解と厳密一致する。
#[test]
fn table_insert_overwrite_oracle() {
    let entries = sample_entries();
    let t = table_of(&entries);
    let got = atom_map(t.iter().map(|(f, v)| (f, *v)), 2);
    assert_eq!(got, oracle(&entries, 2));
}

/// Map: insert（後勝ち）のアトム写像が正解と厳密一致する。
#[test]
fn map_insert_overwrite_oracle() {
    let entries = sample_entries();
    let m = map_of(&entries);
    let got = atom_map(m.iter().map(|(f, v)| (f, *v)), 2);
    assert_eq!(got, oracle(&entries, 2));
}

/// Table: get（時空間クエリ）: 結果 ＝ 全体 ∩ クエリ領域。
#[test]
fn table_get_atom_oracle() {
    let entries = sample_entries();
    let t = table_of(&entries);
    let all = atom_map(t.iter().map(|(f, v)| (f, *v)), 2);
    // クエリ: 粗い空間セル (zoom1) × [60,120)
    let query = FlexId::new(1u8, 0, 1u8, 0, 1u8, 0)
        .map(|id| id.with_temporal(TemporalId::new(60_u64, 1).unwrap()))
        .unwrap();
    let got = atom_map(t.get(&query).map(|(f, v)| (f, *v)), 2);
    let q_keys: alloc::collections::BTreeSet<Key> = spatial_keys(&query, 2)
        .into_iter()
        .flat_map(|k| (60u64..120).map(move |s| (k, s)))
        .collect();
    let exp: BTreeMap<Key, i32> = all
        .iter()
        .filter(|(k, _)| q_keys.contains(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    assert_eq!(got, exp);
}

/// Table: remove: 削除分 ＝ 元 ∩ クエリ、残り ＝ 元 − クエリ（値保持）。
#[test]
fn table_remove_atom_oracle() {
    let entries = sample_entries();
    let mut t = table_of(&entries);
    let before = atom_map(t.iter().map(|(f, v)| (f, *v)), 2);
    let query = make_flex_id(2, 0, 0, 0, 60, 1); // (0,0,0) @ [60,120)
    let removed = atom_map(t.remove(&query), 2);
    let q_keys: alloc::collections::BTreeSet<Key> = (60u64..120).map(|s| ((0, 0, 0), s)).collect();
    let exp_removed: BTreeMap<Key, i32> = before
        .iter()
        .filter(|(k, _)| q_keys.contains(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    let exp_rest: BTreeMap<Key, i32> = before
        .iter()
        .filter(|(k, _)| !q_keys.contains(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    assert_eq!(removed, exp_removed, "removed");
    assert_eq!(
        atom_map(t.iter().map(|(f, v)| (f, *v)), 2),
        exp_rest,
        "rest"
    );
}

/// Map: remove の同正解。
#[test]
fn map_remove_atom_oracle() {
    let entries = sample_entries();
    let mut m = map_of(&entries);
    let before = atom_map(m.iter().map(|(f, v)| (f, *v)), 2);
    let query = make_flex_id(2, 0, 0, 0, 60, 1);
    let removed = atom_map(m.remove(&query), 2);
    let q_keys: alloc::collections::BTreeSet<Key> = (60u64..120).map(|s| ((0, 0, 0), s)).collect();
    let exp_removed: BTreeMap<Key, i32> = before
        .iter()
        .filter(|(k, _)| q_keys.contains(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    let exp_rest: BTreeMap<Key, i32> = before
        .iter()
        .filter(|(k, _)| !q_keys.contains(k))
        .map(|(&k, &v)| (k, v))
        .collect();
    assert_eq!(removed, exp_removed, "removed");
    assert_eq!(
        atom_map(m.iter().map(|(f, v)| (f, *v)), 2),
        exp_rest,
        "rest"
    );
}

/// value_get / rebuild_index: 値 → 時空間の逆引きが4次元で正しい。
#[test]
fn table_value_get_is_spatio_temporal() {
    let entries = sample_entries();
    let mut t = table_of(&entries);
    let all = atom_map(t.iter().map(|(f, v)| (f, *v)), 2);

    // 走査経路（index 未構築）と index 経路の両方を確認する。
    for rebuilt in [false, true] {
        if rebuilt {
            t.rebuild_index();
        }
        for target in [1, 2, 3, 4, 5] {
            let got = atom_map(t.value_get(&target).map(|f| (f, target)), 2);
            let exp: BTreeMap<Key, i32> = all
                .iter()
                .filter(|(_, v)| **v == target)
                .map(|(&k, &v)| (k, v))
                .collect();
            assert_eq!(got, exp, "value_get({target}) rebuilt={rebuilt}");
        }
    }
}

/// 時間 WHOLE のみの利用では、iter が従来どおり temporal=WHOLE で返る（後方互換）。
#[test]
fn spatial_only_data_stays_whole() {
    let mut t = SpatialIdTable::new();
    t.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 7);
    let items: Vec<(FlexId, i32)> = t.iter().map(|(f, v)| (f, *v)).collect();
    assert_eq!(items.len(), 1);
    assert!(items[0].0.temporal().is_whole());
    assert_eq!(items[0].1, 7);

    let mut m = SpatialIdMap::new();
    m.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 7);
    let items: Vec<(FlexId, i32)> = m.iter().map(|(f, v)| (f, *v)).collect();
    assert_eq!(items.len(), 1);
    assert!(items[0].0.temporal().is_whole());
}

/// 時間 WHOLE の値を有限時間で部分上書きしても、残り時間の値が保持される。
#[test]
fn whole_value_partially_overwritten_keeps_rest() {
    let mut t = SpatialIdTable::new();
    t.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap(), 1); // (0,0,0) @ WHOLE = 1
    t.insert(make_flex_id(2, 0, 0, 0, 60, 0), 2); // [0,60) だけ 2

    // [0,60) は 2
    let q = make_flex_id(2, 0, 0, 0, 60, 0);
    let got: Vec<i32> = t.get(&q).map(|(_, v)| *v).collect();
    assert!(!got.is_empty() && got.iter().all(|v| *v == 2));

    // [60,120) は 1 のまま
    let q = make_flex_id(2, 0, 0, 0, 60, 1);
    let got: Vec<i32> = t.get(&q).map(|(_, v)| *v).collect();
    assert!(!got.is_empty() && got.iter().all(|v| *v == 1));

    // 全時間の被覆は保たれている（WHOLE 全長）
    let total: u64 = t
        .iter()
        .map(|(f, _)| f.temporal().end_unixtime_exclusive() - f.temporal().start_unixtime())
        .sum();
    assert_eq!(total, Interval::WHOLE_SECONDS);
}

/// 同値・同時間の空間セルは従来どおりマージされる（圧縮の退行なし）。
#[test]
fn same_value_same_time_cells_still_merge() {
    let mut t = SpatialIdTable::new();
    for f in 0..2 {
        for x in 0..2 {
            for y in 0..2 {
                t.insert(make_flex_id(1, f, x, y, 3600, 7), 9);
            }
        }
    }
    assert_eq!(t.count(), 1, "8 octants (same value & time) merge into 1");
}
