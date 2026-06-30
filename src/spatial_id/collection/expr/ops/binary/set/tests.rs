use crate::SpatialIdCollection;
use crate::{ConflictPolicy, FlexId, SingleId, SpatialIdSet, SpatialIdTable};

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
        .into_query()
        .union_with(b_table(), ConflictPolicy::Max)
        .run()
        .unwrap();

    assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1)); // a のみ
    assert_eq!(value_at(&u, 25, 1, 100, 100), Some(9)); // both → max(2,9)
    assert_eq!(value_at(&u, 25, 2, 100, 100), Some(3)); // b のみ
}

#[test]
fn intersection_keeps_overlap_only() {
    let i = a_table()
        .into_query()
        .intersection_with(b_table(), ConflictPolicy::Min)
        .run()
        .unwrap();

    assert_eq!(value_at(&i, 25, 0, 100, 100), None);
    assert_eq!(value_at(&i, 25, 1, 100, 100), Some(2)); // both → min(2,9)
    assert_eq!(value_at(&i, 25, 2, 100, 100), None);
}

#[test]
fn difference_removes_overlap() {
    let d = a_table().into_query().difference(b_table()).run().unwrap();

    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1)); // a のみ → 残る
    assert_eq!(value_at(&d, 25, 1, 100, 100), None); // 重なり → 削る
    assert_eq!(value_at(&d, 25, 2, 100, 100), None); // b 側は無視
}

#[test]
fn difference_accepts_table_as_mask() {
    let mut mask: SpatialIdTable<u8> = SpatialIdTable::new();
    mask.insert(id(25, 1, 100, 100), 0);

    let d = a_table().into_query().difference(mask).run().unwrap();
    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1));
    assert_eq!(value_at(&d, 25, 1, 100, 100), None);
}

#[test]
fn symmetric_difference_keeps_exclusive_cells() {
    let x = a_table()
        .into_query()
        .symmetric_difference(b_table())
        .run()
        .unwrap();

    assert_eq!(value_at(&x, 25, 0, 100, 100), Some(1)); // a のみ
    assert_eq!(value_at(&x, 25, 1, 100, 100), None); // 重なり → 削る
    assert_eq!(value_at(&x, 25, 2, 100, 100), Some(3)); // b のみ
}

#[test]
fn mask_keeps_left_value_on_overlap() {
    let mut region: SpatialIdTable<u8> = SpatialIdTable::new();
    region.insert(id(25, 1, 100, 100), 0);

    let m = a_table().into_query().mask(region).run().unwrap();
    assert_eq!(value_at(&m, 25, 0, 100, 100), None); // 範囲外 → 落ちる
    assert_eq!(value_at(&m, 25, 1, 100, 100), Some(2)); // a の値を保持
}

#[test]
fn empty_identities() {
    let a = a_table();
    let empty: SpatialIdTable<u8> = SpatialIdTable::new();

    // A ∪ ∅ = A
    let u = a
        .clone()
        .into_query()
        .union_with(empty.clone(), ConflictPolicy::Max)
        .run()
        .unwrap();
    assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1));
    assert_eq!(value_at(&u, 25, 1, 100, 100), Some(2));

    // A ∩ ∅ = ∅
    assert!(
        a.clone()
            .into_query()
            .intersection_with(empty.clone(), ConflictPolicy::Max)
            .run()
            .unwrap()
            .is_empty()
    );

    // A ∖ ∅ = A
    assert_eq!(
        value_at(
            &a.clone()
                .into_query()
                .difference(empty.clone())
                .run()
                .unwrap(),
            25,
            0,
            100,
            100
        ),
        Some(1)
    );

    // A ∖ A = ∅
    assert!(
        a.clone()
            .into_query()
            .difference(a_table().clone())
            .run()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn difference_splits_coarse_cell() {
    // A は x が粗いセル（z24 index50 = z25 の x=100,101 を覆う）。B は内側の1セル(x=100)。
    // A ∖ B は残り（x=101）を細分して返す。
    let mut a: SpatialIdTable<bool> = SpatialIdTable::new();
    a.insert(FlexId::new(25, 0, 24, 50, 25, 100).unwrap(), true);
    let mut b: SpatialIdTable<bool> = SpatialIdTable::new();
    b.insert(id(25, 0, 100, 100), true);

    let d = a.clone().into_query().difference(b.clone()).run().unwrap();
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

    let d = a.clone().into_query().difference(b.clone()).run().unwrap();
    assert!(d.get(&id(25, 0, 100, 100)).next().is_some());
    assert!(d.get(&id(25, 1, 100, 100)).next().is_none());
}
