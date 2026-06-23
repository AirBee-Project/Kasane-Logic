//! Morton バックエンド固有の挙動（直接の演算子経路・入れ子・正規化）を検証する。

use crate::{SingleId, SpatialIdSet};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

fn s(z: u8, f: i32, x: u32, y: u32) -> SingleId {
    SingleId::new(z, f, x, y).unwrap()
}

fn set_of(ids: &[SingleId]) -> SpatialIdSet {
    let mut set = SpatialIdSet::new();
    for id in ids {
        set.insert(id.clone());
    }
    set
}

/// 集合を最大ズームへ正規化した単一IDの並びへ落として比較しやすくする。
fn flat(set: &SpatialIdSet) -> BTreeSet<SingleId> {
    set.flat_single_ids().collect()
}

#[test]
fn insert_count_and_get_uniform() {
    let set = set_of(&[s(20, 0, 0, 0), s(20, 0, 1, 0), s(20, 0, 1, 0)]);
    // 同一セルの二重挿入は 1 つ。
    assert_eq!(set.count(), 2);

    let hit: Vec<_> = set.get(&s(20, 0, 1, 0)).collect();
    assert_eq!(hit, vec![s(20, 0, 1, 0)]);

    let miss: Vec<_> = set.get(&s(20, 0, 9, 9)).collect();
    assert!(miss.is_empty());
}

#[test]
fn union_uniform_fast_path() {
    let a = set_of(&[s(20, 0, 0, 0), s(20, 0, 1, 0)]);
    let b = set_of(&[s(20, 0, 1, 0), s(20, 0, 2, 0)]);
    let u = &a | &b;
    assert_eq!(
        flat(&u),
        BTreeSet::from([s(20, 0, 0, 0), s(20, 0, 1, 0), s(20, 0, 2, 0)])
    );
}

#[test]
fn intersection_uniform_fast_path() {
    let a = set_of(&[s(20, 0, 0, 0), s(20, 0, 1, 0)]);
    let b = set_of(&[s(20, 0, 1, 0), s(20, 0, 2, 0)]);
    let i = &a & &b;
    assert_eq!(flat(&i), BTreeSet::from([s(20, 0, 1, 0)]));
}

#[test]
fn difference_uniform_fast_path() {
    let a = set_of(&[s(20, 0, 0, 0), s(20, 0, 1, 0)]);
    let b = set_of(&[s(20, 0, 1, 0), s(20, 0, 2, 0)]);
    let d = &a - &b;
    assert_eq!(flat(&d), BTreeSet::from([s(20, 0, 0, 0)]));
}

#[test]
fn nested_insert_splits_ancestor() {
    // 粗い z1 セルを入れた後、その内側の z2 セルを入れると z2 解像度の 8 セルへ分割される。
    let mut set = SpatialIdSet::new();
    set.insert(s(1, 0, 0, 0));
    set.insert(s(2, 0, 0, 0));
    assert_eq!(set.count(), 8, "ancestor must split into 8 children");

    // 覆われた領域は依然として全て get で当たる。
    for child in s(1, 0, 0, 0).spatial_children_at_zoom(2).unwrap() {
        assert_eq!(
            set.get(&child).count(),
            1,
            "child {child:?} should be present"
        );
    }
}

#[test]
fn equality_is_resolution_independent() {
    // 粗い 1 セルと、その 8 子セルは同じ領域なので等しい。
    let coarse = set_of(&[s(1, 0, 0, 0)]);
    let children: Vec<SingleId> = s(1, 0, 0, 0).spatial_children_at_zoom(2).unwrap().collect();
    let fine = set_of(&children);
    assert_eq!(coarse, fine);
}

#[test]
fn intersection_mixed_zoom_general_path() {
    // A は粗い z1 セル、B はその内側の 1 つの z2 セル → 積はその z2 セル。
    let a = set_of(&[s(1, 0, 0, 0)]);
    let b = set_of(&[s(2, 0, 0, 0)]);
    let i = &a & &b;
    assert_eq!(flat(&i), flat(&b));
}

#[test]
fn difference_mixed_zoom_general_path() {
    // 粗い z1 セルから内側の z2 セル 1 つを引くと、残り 7 つの z2 セル。
    let a = set_of(&[s(1, 0, 0, 0)]);
    let b = set_of(&[s(2, 0, 0, 0)]);
    let d = &a - &b;
    assert_eq!(d.count(), 7);

    let expected: BTreeSet<SingleId> = s(1, 0, 0, 0)
        .spatial_children_at_zoom(2)
        .unwrap()
        .filter(|c| *c != s(2, 0, 0, 0))
        .collect();
    assert_eq!(flat(&d), expected);
}

#[test]
fn remove_clips_region() {
    let mut set = set_of(&[s(20, 0, 0, 0), s(20, 0, 1, 0)]);
    let removed: Vec<_> = set.remove(&s(20, 0, 0, 0)).collect();
    assert_eq!(removed, vec![s(20, 0, 0, 0)]);
    assert_eq!(set.count(), 1);
    assert_eq!(set.get(&s(20, 0, 0, 0)).count(), 0);
}

#[test]
fn get_overlapping_returns_stored_cell_unclipped() {
    // 粗い z1 セルを保持し、内側の z3 セルで問い合わせると、格納済みの粗いセルが返る。
    let set = set_of(&[s(1, 0, 0, 0)]);
    let probe = s(3, 0, 0, 0); // z1 セル内の点
    let overlapping: Vec<_> = set.get_overlapping(&probe).collect();
    assert_eq!(overlapping, vec![s(1, 0, 0, 0)]);
}
