//! シャード初期化（[`SpatialIdSet::new_in_shard`]）の挙動テスト。
//!
//! - 領域外への挿入は無視される
//! - 領域をまたぐ粗い挿入は領域へ切り詰められる
//! - 領域が交わらない集合同士の演算は早期に確定する

#![cfg(test)]

#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, SingleId, SpatialIdSet};

/// ズーム `z` の単一セルを表す [`FlexId`] 領域を作る。
fn region(z: u8, f: i32, x: u32, y: u32) -> FlexId {
    FlexId::from(SingleId::new(z, f, x, y).unwrap())
}

#[test]
fn insert_inside_region_is_kept() {
    // ズーム2のタイル (0,0,0) をシャード領域にする。
    let mut set = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    // 領域内のより細かいセル。
    let inside = SingleId::new(4, 0, 1, 1).unwrap();
    set.insert(inside.clone());

    assert_eq!(set.count(), 1);
    // 取り出した結果が入力と一致する（切り詰めなし）。
    let got: Vec<FlexId> = set.iter().collect();
    assert_eq!(got, vec![FlexId::from(inside)]);
}

#[test]
fn insert_outside_region_is_ignored() {
    let mut set = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    // 同じズーム2の別タイル → 領域外。
    set.insert(SingleId::new(2, 0, 3, 3).unwrap());

    assert!(set.is_empty());
    assert_eq!(set.count(), 0);
}

#[test]
fn coarse_insert_is_clipped_to_region() {
    let shard = region(2, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard.clone());
    // ズーム0（全空間）を挿入 → 領域に切り詰められるはず。
    set.insert(SingleId::new(0, 0, 0, 0).unwrap());

    let got: Vec<FlexId> = set.iter().collect();
    assert_eq!(got, vec![shard]);
}

#[test]
fn intersection_of_disjoint_shards_is_empty() {
    let mut a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());

    let mut b = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    b.insert(SingleId::new(4, 0, 13, 13).unwrap());

    // 領域が交わらない → 交差は空（早期確定）。
    let inter = &a & &b;
    assert!(inter.is_empty());
}

#[test]
fn difference_of_disjoint_shards_is_lhs() {
    let mut a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());

    let mut b = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    b.insert(SingleId::new(4, 0, 13, 13).unwrap());

    // 領域が交わらない → 差は lhs そのまま（早期確定）。
    let diff = &a - &b;
    assert_eq!(diff, a);
}

#[test]
fn same_region_intersection_matches_overlap() {
    let shard = region(2, 0, 0, 0);
    let mut a = SpatialIdSet::new_in_shard(shard.clone());
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());
    a.insert(SingleId::new(4, 0, 2, 2).unwrap());

    let mut b = SpatialIdSet::new_in_shard(shard);
    b.insert(SingleId::new(4, 0, 2, 2).unwrap());

    let inter = &a & &b;
    let got: Vec<FlexId> = inter.iter().collect();
    assert_eq!(got, vec![FlexId::from(SingleId::new(4, 0, 2, 2).unwrap())]);
}

#[test]
fn split_shard_then_merge_shards_roundtrips() {
    // Map と対称な split_shard / merge_shards が、分割→統合で元の集合に戻ることを確認。
    let shard = region(2, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard.clone());
    set.insert(SingleId::new(4, 0, 1, 1).unwrap());
    set.insert(SingleId::new(4, 0, 2, 2).unwrap());
    set.insert(SingleId::new(4, 0, 3, 0).unwrap());

    let ((lr, lower), (ur, upper)) = set.split_shard().expect("sharded set must split");

    // 2子のシャード領域は親に内包される（被覆分割の不変条件）。
    assert_eq!(shard.intersection(&lr).as_ref(), Some(&lr));
    assert_eq!(shard.intersection(&ur).as_ref(), Some(&ur));

    // 統合すると領域・内容ともに元へ戻る。
    let merged = SpatialIdSet::merge_shards(shard.clone(), vec![lower, upper]).unwrap();
    assert_eq!(merged.shard(), Some(&shard));
    assert_eq!(merged, set);
}

#[test]
fn merge_shards_rejects_region_outside_parent() {
    // 親領域に内包されない子を渡すと InvalidShardMerge。
    let parent = region(2, 0, 0, 0);
    let outside = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    assert!(SpatialIdSet::merge_shards(parent, vec![outside]).is_err());
}

#[test]
fn merge_shards_rejects_overlapping_children() {
    // 親には内包されるが、子同士が重なる（同一領域）→ 互いに素でないので拒否。
    let parent = region(1, 0, 0, 0);
    let a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    let b = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    assert!(SpatialIdSet::merge_shards(parent, vec![a, b]).is_err());
}

#[test]
fn merge_shards_rejects_shardless_child() {
    // シャード領域未設定（new()）の子は検証不能なので拒否。
    let parent = region(1, 0, 0, 0);
    let shardless = SpatialIdSet::new();
    assert!(SpatialIdSet::merge_shards(parent, vec![shardless]).is_err());
}
