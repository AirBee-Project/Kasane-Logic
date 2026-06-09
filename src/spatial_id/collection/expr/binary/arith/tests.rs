#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{Addable, Multipliable, SingleId, SpatialIdTable, Subtractable};

fn id(z: u8, f: i32, x: u32, y: u32) -> SingleId {
    SingleId::new(z, f, x, y).unwrap()
}

fn value_at(t: &SpatialIdTable<i32>, z: u8, f: i32, x: u32, y: u32) -> Option<i32> {
    t.get(&id(z, f, x, y)).next().map(|(_, v)| *v)
}

// 共通の入力: a = {f0:10, f1:20}, b = {f1:5, f2:3}（f1 で重なる）。
fn a_table() -> SpatialIdTable<i32> {
    let mut a = SpatialIdTable::new();
    a.insert(id(25, 0, 100, 100), 10);
    a.insert(id(25, 1, 100, 100), 20);
    a
}

fn b_table() -> SpatialIdTable<i32> {
    let mut b = SpatialIdTable::new();
    b.insert(id(25, 1, 100, 100), 5);
    b.insert(id(25, 2, 100, 100), 3);
    b
}

#[test]
fn add_sums_overlap_and_keeps_each_side() {
    let s = a_table().add(&b_table()).unwrap();

    assert_eq!(value_at(&s, 25, 0, 100, 100), Some(10)); // a のみ
    assert_eq!(value_at(&s, 25, 1, 100, 100), Some(25)); // both → 20 + 5
    assert_eq!(value_at(&s, 25, 2, 100, 100), Some(3)); // b のみ
}

#[test]
fn add_is_commutative() {
    let ab = a_table().add(&b_table()).unwrap();
    let ba = b_table().add(&a_table()).unwrap();

    for f in 0..=2 {
        assert_eq!(
            value_at(&ab, 25, f, 100, 100),
            value_at(&ba, 25, f, 100, 100)
        );
    }
}

#[test]
fn add_with_empty_is_identity() {
    let empty = SpatialIdTable::<i32>::new();
    let s = a_table().add(&empty).unwrap();

    assert_eq!(value_at(&s, 25, 0, 100, 100), Some(10));
    assert_eq!(value_at(&s, 25, 1, 100, 100), Some(20));
    assert_eq!(value_at(&s, 25, 2, 100, 100), None);
}

#[test]
fn sub_keeps_a_domain_and_drops_b_only() {
    let d = a_table().sub(&b_table()).unwrap();

    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(10)); // a のみ → a
    assert_eq!(value_at(&d, 25, 1, 100, 100), Some(15)); // both → 20 - 5
    assert_eq!(value_at(&d, 25, 2, 100, 100), None); // b のみ → 捨てる
}

#[test]
fn sub_self_is_zero_over_a_domain() {
    let d = a_table().sub(&a_table()).unwrap();

    assert_eq!(value_at(&d, 25, 0, 100, 100), Some(0));
    assert_eq!(value_at(&d, 25, 1, 100, 100), Some(0));
}

#[test]
fn mul_keeps_overlap_only() {
    let m = a_table().mul(&b_table()).unwrap();

    assert_eq!(value_at(&m, 25, 0, 100, 100), None); // a のみ → 捨てる
    assert_eq!(value_at(&m, 25, 1, 100, 100), Some(100)); // both → 20 * 5
    assert_eq!(value_at(&m, 25, 2, 100, 100), None); // b のみ → 捨てる
}

#[test]
fn mul_is_commutative() {
    let ab = a_table().mul(&b_table()).unwrap();
    let ba = b_table().mul(&a_table()).unwrap();

    for f in 0..=2 {
        assert_eq!(
            value_at(&ab, 25, f, 100, 100),
            value_at(&ba, 25, f, 100, 100)
        );
    }
}

#[test]
fn mul_over_overlapping_ranges_at_mixed_zoom() {
    // 粗いセル（z24）に細かいセル（z25）が重なるケース。重なり部分のみ積が残る。
    let mut a = SpatialIdTable::<i32>::new();
    a.insert(id(24, 0, 50, 50), 3); // z25 の f0,f1 を覆う粗いセル

    let mut b = SpatialIdTable::<i32>::new();
    b.insert(id(25, 0, 100, 100), 7); // a の被覆領域内の細かいセル

    let m = a.mul(&b).unwrap();

    // 重なるセルは 3 * 7、覆われていない残りは片側のみなので消える。
    assert_eq!(value_at(&m, 25, 0, 100, 100), Some(21));
    assert_eq!(value_at(&m, 25, 1, 100, 100), None);
}

#[test]
fn add_over_overlapping_ranges_at_mixed_zoom() {
    // 粗いセル（z24）に細かいセル（z25）が重なるケース。重なり部分のみ加算される。
    let mut a = SpatialIdTable::<i32>::new();
    a.insert(id(24, 0, 50, 50), 100); // z25 の f0,f1 を覆う粗いセル

    let mut b = SpatialIdTable::<i32>::new();
    b.insert(id(25, 0, 100, 100), 1); // a の被覆領域内の細かいセル

    let s = a.add(&b).unwrap();

    // 重なるセルは 100 + 1、覆われていない残りは a の 100 のまま。
    assert_eq!(value_at(&s, 25, 0, 100, 100), Some(101));
    assert_eq!(value_at(&s, 25, 1, 100, 100), Some(100));
}
