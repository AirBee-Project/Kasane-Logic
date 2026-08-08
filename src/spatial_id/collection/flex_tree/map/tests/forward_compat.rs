//! 永続化形式の前方互換性と、バイト列へ焼いた要約・拡張領域の検証。
//!
//! ここが守っているのは「リリース後に値を足しても既存データが読めること」である。
//! [`FORMAT_VERSION`](crate::FORMAT_VERSION) を上げずに拡張できる唯一の経路
//! （`MapArena::ext` の TLV）が生きているかを機械的に確かめる。

#![cfg(all(test, feature = "persist"))]

use crate::{
    ArchivedSpatialIdMap, FlexId, RangeId, ShardPath, Side, SingleId, SpatialIdMap, SpatialIdSet,
    push_ext_entry,
};
use alloc::vec;
use alloc::vec::Vec;

fn sample() -> SpatialIdMap<Vec<u8>> {
    let mut m = SpatialIdMap::new();
    m.insert(SingleId::new(4, 0, 0, 0).unwrap(), vec![0xAA]);
    m.insert(SingleId::new(6, 3, 9, 9).unwrap(), vec![0xBB]);
    m.insert(SingleId::new(4, -1, 1, 1).unwrap(), vec![0xCC]);
    m
}

/// バイト列へ焼いた要約が、インメモリで計算した要約と完全に一致すること。
///
/// ここがずれると、シャードを読まずに行う枝刈りが**間違った枝を落とす**。
#[test]
fn persisted_summary_matches_the_in_memory_one() {
    let map = sample();
    let bytes = map.to_bytes().unwrap();
    let archived = unsafe { ArchivedSpatialIdMap::access(&bytes) }.unwrap();

    assert_eq!(archived.summary(), map.summary());
}

/// 要約の bounding box が、既存の `bounding_box()` と一致すること。
#[test]
fn summary_bbox_matches_bounding_box() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(4, 0, 0, 0).unwrap());
    set.insert(SingleId::new(7, 5, 40, 40).unwrap());
    set.insert(SingleId::new(4, -1, 1, 1).unwrap());

    assert_eq!(set.summary().bbox().cloned(), set.bounding_box());
}

/// 空のマップの要約が「空」を表すこと（枝刈りで必ず落ちる）。
#[test]
fn empty_map_summary_never_intersects() {
    let map: SpatialIdMap<Vec<u8>> = SpatialIdMap::new();
    let summary = map.summary();

    assert!(summary.is_empty());
    assert_eq!(summary.bbox(), None);
    assert_eq!(summary.seconds_range(), None);
    assert!(!summary.intersects(&RangeId::new(0, [0, 0], [0, 0], [0, 0]).unwrap()));
}

/// 要約による枝刈りに**偽陰性が無い**こと。
///
/// 実際に交差するSegmentが1つでもあるなら `intersects` は必ず真でなければならない
/// （偽陽性は許される）。ここが破れると、読むべきシャードを読まずに答えを返す。
#[test]
fn summary_pruning_has_no_false_negatives() {
    let mut set = SpatialIdSet::new();
    set.insert(SingleId::new(5, 2, 10, 10).unwrap());
    set.insert(SingleId::new(5, 3, 20, 20).unwrap());
    let summary = set.summary();

    // 走査対象をひととおり動かして、実際の交差と要約の判定を突き合わせる。
    for f in 0..6i32 {
        for x in 8..24u32 {
            for y in 8..24u32 {
                let target = RangeId::new(5, [f, f], [x, x], [y, y]).unwrap();
                let really_intersects = set.iter().any(|id| id.intersects_range(&target));
                if really_intersects {
                    assert!(
                        summary.intersects(&target),
                        "交差するのに要約が枝刈りした: {target:?}"
                    );
                }
            }
        }
    }
}

/// 拡張領域が往復し、未知タグは読み飛ばされること。
#[test]
fn ext_entries_round_trip_and_unknown_tags_are_skipped() {
    let mut ext = Vec::new();
    push_ext_entry(&mut ext, 1, b"first");
    push_ext_entry(&mut ext, 0x9000, b"");
    push_ext_entry(&mut ext, 42, b"third");

    let bytes = sample().to_bytes_with_ext(&ext).unwrap();
    let archived = unsafe { ArchivedSpatialIdMap::access(&bytes) }.unwrap();

    assert_eq!(archived.ext(1), Some(&b"first"[..]));
    assert_eq!(archived.ext(0x9000), Some(&b""[..]));
    assert_eq!(archived.ext(42), Some(&b"third"[..]));
    // 知らないタグは「無い」だけで、読み出し自体は成功する。
    assert_eq!(archived.ext(7), None);

    assert_eq!(archived.ext_entries().count(), 3);
}

/// 壊れた拡張領域が本体の読み出しを妨げないこと。
///
/// 拡張領域は「後から足すための緩衝地帯」なので、そこが壊れても木は読めなければならない。
#[test]
fn malformed_ext_does_not_break_the_body() {
    let mut ext = Vec::new();
    push_ext_entry(&mut ext, 1, b"ok");
    // 長さだけ巨大で中身が無いエントリを継ぎ足す。
    ext.extend_from_slice(&2u16.to_le_bytes());
    ext.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());

    let map = sample();
    let bytes = map.to_bytes_with_ext(&ext).unwrap();
    let archived = unsafe { ArchivedSpatialIdMap::access(&bytes) }.unwrap();

    // 壊れる前のエントリは読める。壊れた先は打ち切られる。
    assert_eq!(archived.ext(1), Some(&b"ok"[..]));
    assert_eq!(archived.ext(2), None);
    // 本体と要約は無事。
    assert_eq!(archived.summary(), map.summary());

    let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes) }.unwrap();
    assert_eq!(restored.count(), map.count());
}

/// シャード分割がパスを1段ずつ記録し、それがバイト列を往復すること。
#[test]
fn shard_path_is_recorded_by_split_and_survives_a_round_trip() {
    let mut set = SpatialIdSet::new_in_shard(FlexId::UPPER_MAX);
    for x in 0..8u32 {
        set.insert(SingleId::new(4, 1, x, 1).unwrap());
    }

    assert_eq!(set.shard_path(), Some(&ShardPath::root(true)));

    let ((_, lower), (_, upper)) = set.split_shard().unwrap();
    assert_eq!(
        lower.shard_path(),
        Some(&ShardPath::root(true).child(Side::Lower))
    );
    assert_eq!(
        upper.shard_path(),
        Some(&ShardPath::root(true).child(Side::Upper))
    );

    // 孫まで降りてもパスが伸びること。
    let ((_, grandchild), _) = lower.split_shard().unwrap();
    let expected = ShardPath::root(true).child(Side::Lower).child(Side::Lower);
    assert_eq!(grandchild.shard_path(), Some(&expected));

    // バイト列を往復してもパスが保たれること。
    let mut map: SpatialIdMap<Vec<u8>> = SpatialIdMap::new_in_shard(FlexId::UPPER_MAX);
    map.insert(SingleId::new(4, 1, 1, 1).unwrap(), vec![1]);
    let ((_, lower_map), _) = map.split_shard().unwrap();

    let bytes = lower_map.to_bytes().unwrap();
    let archived = unsafe { ArchivedSpatialIdMap::access(&bytes) }.unwrap();
    assert_eq!(archived.shard_path(), lower_map.shard_path().cloned());

    let restored = unsafe { SpatialIdMap::<Vec<u8>>::from_bytes(&bytes) }.unwrap();
    assert_eq!(restored.shard_path(), lower_map.shard_path());
}

/// 分割で得た子シャードのキーが、親のキーを接頭辞に持つこと。
///
/// KVS のプレフィックススキャンで「ある領域の配下のシャードを全部取る」ための前提。
#[test]
fn child_shard_keys_extend_the_parent_key() {
    let mut set = SpatialIdSet::new_in_shard(FlexId::UPPER_MAX);
    for x in 0..8u32 {
        set.insert(SingleId::new(4, 1, x, 1).unwrap());
    }
    let parent_key = set.shard_path().unwrap().key();

    let ((_, lower), (_, upper)) = set.split_shard().unwrap();
    let lower_key = lower.shard_path().unwrap().key();
    let upper_key = upper.shard_path().unwrap().key();

    assert!(lower_key.starts_with(parent_key));
    assert!(upper_key.starts_with(parent_key));
    assert!(lower_key < upper_key, "兄弟が Lower → Upper の順でない");
}

/// 集合演算の結果はシャード木のノードとは限らないので、位置が違えばパスを引き継がないこと。
#[test]
fn set_algebra_drops_a_mismatched_shard_path() {
    let mut a = SpatialIdSet::new_in_shard(FlexId::UPPER_MAX);
    a.insert(SingleId::new(4, 1, 1, 1).unwrap());
    let ((_, a_lower), (_, a_upper)) = a.split_shard().unwrap();

    // 同じ位置同士なら引き継ぐ。
    assert_eq!(
        (&a_lower | &a_lower).shard_path(),
        a_lower.shard_path(),
        "同一位置の union でパスが失われた"
    );
    // 違う位置同士なら位置不明へ落ちる。
    assert_eq!((&a_lower | &a_upper).shard_path(), None);
}
