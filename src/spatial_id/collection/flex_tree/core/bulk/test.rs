//! 一括構築が逐次 `insert` と同じ木（FXY-正規形は一意）を返すことの検証。

use super::{SingleEntry, sort_and_dedup};
use crate::spatial_id::collection::flex_tree::core::{FlexTreeCore, SafeValue};
use crate::{FlexId, SingleId};
use alloc::vec::Vec;

/// 逐次 `insert_with` で組んだ参照実装。
fn reference<V, R>(z: u8, entries: &[SingleEntry<V>], resolve: &R) -> FlexTreeCore<V>
where
    V: SafeValue,
    R: Fn(&V, &V) -> V + Sync,
{
    let mut core = FlexTreeCore::new();
    for (f, x, y, v) in entries {
        core.insert_with(
            FlexId::new(z, *f, z, *x, z, *y).unwrap(),
            v.clone(),
            resolve,
        );
    }
    core
}

fn max_u32(a: &u32, b: &u32) -> u32 {
    *a.max(b)
}

fn check(z: u8, mut entries: Vec<SingleEntry<u32>>) {
    let expected = reference(z, &entries, &max_u32);
    sort_and_dedup(&mut entries, &max_u32);
    let got = FlexTreeCore::from_uniform_single_ids(z, &entries);

    got.assert_canonical();
    assert_eq!(got.count(), expected.count(), "葉数が一致しない (z={z})");

    let mut a: Vec<(FlexId, u32)> = got.iter().collect();
    let mut b: Vec<(FlexId, u32)> = expected.iter().collect();
    a.sort();
    b.sort();
    assert_eq!(a, b, "内容が一致しない (z={z})");
    assert!(got == expected, "木の構造が一致しない (z={z})");
}

#[test]
fn single_entry() {
    check(4, alloc::vec![(0, 0, 0, 1)]);
    check(4, alloc::vec![(-1, 3, 5, 7)]);
}

#[test]
fn zoom_zero() {
    check(0, alloc::vec![(0, 0, 0, 1)]);
    check(0, alloc::vec![(0, 0, 0, 1), (-1, 0, 0, 2)]);
}

#[test]
fn straddles_both_roots() {
    check(
        3,
        alloc::vec![
            (-8, 0, 0, 1),
            (-1, 7, 7, 2),
            (0, 0, 0, 3),
            (7, 7, 7, 4),
            (3, 2, 1, 5),
        ],
    );
}

/// 同値で埋まった立方体は 1 枚の葉へ畳まれる（正規形 N1）。
#[test]
fn uniform_block_collapses() {
    let mut entries = Vec::new();
    for f in 0..4i32 {
        for x in 0..4u32 {
            for y in 0..4u32 {
                entries.push((f, x, y, 9u32));
            }
        }
    }
    let expected = reference(2, &entries, &max_u32);
    sort_and_dedup(&mut entries, &max_u32);
    let got = FlexTreeCore::from_uniform_single_ids(2, &entries);
    assert_eq!(got.count(), 1, "一様な立方体は 1 葉に畳まれるはず");
    assert!(got == expected);
}

/// 1 軸だけ値が変わる板は、その軸だけが分割された異方的な葉になる。
#[test]
fn anisotropic_slabs() {
    let mut entries = Vec::new();
    for f in 0..8i32 {
        for x in 0..8u32 {
            for y in 0..8u32 {
                // x にだけ依存する値 → X 軸だけが分割される
                entries.push((f, x, y, x));
            }
        }
    }
    let expected = reference(3, &entries, &max_u32);
    sort_and_dedup(&mut entries, &max_u32);
    let got = FlexTreeCore::from_uniform_single_ids(3, &entries);
    assert_eq!(got.count(), 8, "X 方向 8 枚の板になるはず");
    assert!(
        got.iter()
            .all(|(id, _)| id.f_zoomlevel() == 0 && id.y_zoomlevel() == 0)
    );
    assert!(got == expected);
}

#[test]
fn duplicates_are_resolved() {
    let entries = alloc::vec![(1, 1, 1, 5u32), (1, 1, 1, 9), (1, 1, 1, 3), (2, 2, 2, 4)];
    let mut sorted = entries.clone();
    sort_and_dedup(&mut sorted, &max_u32);
    assert_eq!(sorted.len(), 2);
    let got = FlexTreeCore::from_uniform_single_ids(3, &sorted);
    assert!(got == reference(3, &entries, &max_u32));
}

/// 疑似乱数で散らした入力でも逐次 insert と一致する。
#[test]
fn random_entries_match_sequential() {
    for z in [1u8, 2, 5, 8, 12] {
        let span = 1i64 << z;
        let mut state = 0x2545F4914F6CDD1Du64 ^ (z as u64);
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut entries: Vec<SingleEntry<u32>> = Vec::new();
        for _ in 0..600 {
            let f = (next() % (2 * span as u64)) as i64 - span;
            let x = (next() % span as u64) as u32;
            let y = (next() % span as u64) as u32;
            entries.push((f as i32, x, y, (next() % 5) as u32));
        }
        check(z, entries);
    }
}

/// 木 → 一様 SingleId 列 → 木 の往復で元の木に戻る（異方な FlexId を含む）。
#[test]
fn round_trip_through_uniform_single_ids() {
    let mut core: FlexTreeCore<u32> = FlexTreeCore::new();
    core.insert(SingleId::new(6, 1, 5, 5).unwrap(), 1);
    core.insert(SingleId::new(8, 12, 40, 41).unwrap(), 2);
    core.insert(SingleId::new(4, -1, 2, 3).unwrap(), 3);

    let z = core.max_zoomlevel().unwrap();
    let mut entries: Vec<SingleEntry<u32>> = Vec::new();
    for (id, v) in core.iter() {
        let nf = 1i32 << (z - id.f_zoomlevel());
        let nx = 1u32 << (z - id.x_zoomlevel());
        let ny = 1u32 << (z - id.y_zoomlevel());
        for df in 0..nf {
            for dx in 0..nx {
                for dy in 0..ny {
                    entries.push((
                        id.f_index() * nf + df,
                        id.x_index() * nx + dx,
                        id.y_index() * ny + dy,
                        v,
                    ));
                }
            }
        }
    }
    sort_and_dedup(&mut entries, &max_u32);
    let rebuilt = FlexTreeCore::from_uniform_single_ids(z, &entries);
    rebuilt.assert_canonical();
    assert!(rebuilt == core, "一様 SingleId 列の往復で木が変わった");
}
