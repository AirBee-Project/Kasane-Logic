#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{ConflictPolicy, FlexId, SetOps, SingleId, SpatialIdSet, SpatialIdTable};

fn id(z: u8, f: i32, x: u32, y: u32) -> SingleId {
    SingleId::new(z, f, x, y).unwrap()
}

fn value_at(t: &SpatialIdTable<u8>, z: u8, f: i32, x: u32, y: u32) -> Option<u8> {
    t.get(&id(z, f, x, y)).next().map(|(_, v)| *v)
}

fn present_b(t: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
    t.get(&id(z, f, x, y)).next().is_some()
}

// 共通の入力: a = {f0:1, f1:2}, b = {f1:9, f2:3}（f1 で重なる）。
fn a_table() -> SpatialIdTable<u8> {
    let mut a = SpatialIdTable::new();
    a.insert(id(25, 0, 100, 100), 1);
    a.insert(id(25, 1, 100, 100), 2);
    a
}

fn b_table() -> SpatialIdTable<u8> {
    let mut b = SpatialIdTable::new();
    b.insert(id(25, 1, 100, 100), 9);
    b.insert(id(25, 2, 100, 100), 3);
    b
}

#[test]
fn union_keeps_both_sides_and_resolves_overlap() {
    let u = a_table()
        .union_with(&b_table(), ConflictPolicy::Max)
        .unwrap();

    assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1)); // a のみ
    assert_eq!(value_at(&u, 25, 1, 100, 100), Some(9)); // both → max(2,9)
    assert_eq!(value_at(&u, 25, 2, 100, 100), Some(3)); // b のみ
}

#[test]
fn intersection_keeps_overlap_only() {
    let i = a_table()
        .intersection_with(&b_table(), ConflictPolicy::Min)
        .unwrap();

    assert_eq!(value_at(&i, 25, 0, 100, 100), None);
    assert_eq!(value_at(&i, 25, 1, 100, 100), Some(2)); // both → min(2,9)
    assert_eq!(value_at(&i, 25, 2, 100, 100), None);
}

#[test]
fn difference_removes_overlap() {
    let d = a_table().difference(&b_table()).unwrap();

    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1)); // a のみ → 残る
    assert_eq!(value_at(&d, 25, 1, 100, 100), None); // 重なり → 削る
    assert_eq!(value_at(&d, 25, 2, 100, 100), None); // b 側は無視
}

#[test]
fn difference_accepts_set_as_mask() {
    // 別型（Set）をマスクに差分。結果は a の値型を保つ。
    let mut mask = SpatialIdSet::new();
    mask.insert(id(25, 1, 100, 100));

    let d = a_table().difference(&mask).unwrap();
    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1));
    assert_eq!(value_at(&d, 25, 1, 100, 100), None);
}

#[test]
fn symmetric_difference_keeps_exclusive_cells() {
    let x = a_table().symmetric_difference(&b_table()).unwrap();

    assert_eq!(value_at(&x, 25, 0, 100, 100), Some(1)); // a のみ
    assert_eq!(value_at(&x, 25, 1, 100, 100), None); // 重なり → 削る
    assert_eq!(value_at(&x, 25, 2, 100, 100), Some(3)); // b のみ
}

#[test]
fn mask_keeps_left_value_on_overlap() {
    let mut region = SpatialIdSet::new();
    region.insert(id(25, 1, 100, 100));

    let m = a_table().mask(&region).unwrap();
    assert_eq!(value_at(&m, 25, 0, 100, 100), None); // 範囲外 → 落ちる
    assert_eq!(value_at(&m, 25, 1, 100, 100), Some(2)); // a の値を保持
}

#[test]
fn combine_with_merges_different_types() {
    // a: Table<i32>, b: Table<bool> を i32 へ合成。
    let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    a.insert(id(25, 0, 100, 100), 10);
    a.insert(id(25, 1, 100, 100), 20);
    let mut b: SpatialIdTable<bool> = SpatialIdTable::new();
    b.insert(id(25, 1, 100, 100), true);
    b.insert(id(25, 2, 100, 100), true);

    let r: SpatialIdTable<i32> = a
        .combine_with(&b, |av, bv| match (av, bv) {
            (Some(a), Some(_)) => Some(a * 2), // both → 倍
            (Some(a), None) => Some(*a),       // a のみ → そのまま
            (None, _) => None,                 // b のみ → 捨てる
        })
        .unwrap();

    let value_i32 = |z, f, x, y| r.get(&id(z, f, x, y)).next().map(|(_, v)| *v);
    assert_eq!(value_i32(25, 0, 100, 100), Some(10)); // a のみ
    assert_eq!(value_i32(25, 1, 100, 100), Some(40)); // both
    assert_eq!(value_i32(25, 2, 100, 100), None); // b のみ → 除外
}

#[test]
fn empty_identities() {
    let a = a_table();
    let empty: SpatialIdTable<u8> = SpatialIdTable::new();

    // A ∪ ∅ = A
    let u = a.union_with(&empty, ConflictPolicy::Max).unwrap();
    assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1));
    assert_eq!(value_at(&u, 25, 1, 100, 100), Some(2));

    // A ∩ ∅ = ∅
    assert!(
        a.intersection_with(&empty, ConflictPolicy::Max)
            .unwrap()
            .is_empty()
    );

    // A ∖ ∅ = A
    assert_eq!(
        value_at(&a.difference(&empty).unwrap(), 25, 0, 100, 100),
        Some(1)
    );

    // A ∖ A = ∅
    assert!(a.difference(&a_table()).unwrap().is_empty());
}

#[test]
fn difference_splits_coarse_cell() {
    // A は x が粗いセル（z24 index50 = z25 の x=100,101 を覆う）。B は内側の1セル(x=100)。
    // A ∖ B は残り（x=101）を細分して返す。
    let mut a: SpatialIdTable<bool> = SpatialIdTable::new();
    a.insert(FlexId::new(25, 0, 24, 50, 25, 100).unwrap(), true);
    let mut b: SpatialIdTable<bool> = SpatialIdTable::new();
    b.insert(id(25, 0, 100, 100), true);

    let d = a.difference(&b).unwrap();
    assert!(present_b(&d, 25, 0, 101, 100)); // 残り
    assert!(!present_b(&d, 25, 0, 100, 100)); // くり抜かれた
}

#[test]
fn works_on_sets() {
    // Set 同士の集合演算（値なし）。
    let mut a = SpatialIdSet::new();
    a.insert(id(25, 0, 100, 100));
    a.insert(id(25, 1, 100, 100));
    let mut b = SpatialIdSet::new();
    b.insert(id(25, 1, 100, 100));

    let d = a.difference(&b).unwrap();
    assert!(d.get(&id(25, 0, 100, 100)).next().is_some());
    assert!(d.get(&id(25, 1, 100, 100)).next().is_none());
}
