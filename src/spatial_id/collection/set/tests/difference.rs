use crate::spatial_id::collection::set::tests::{set_a, set_b, set_c, to_flat_set};
use std::collections::HashSet;

#[test]
fn test_difference_two_sets() {
    let set_a = set_a();
    let set_b = set_b();

    let logic_result = set_a.difference(&set_b);

    let target_z = [set_a.max_z(), set_b.max_z(), logic_result.max_z()]
        .into_iter()
        .max()
        .unwrap();

    let actual = to_flat_set(&logic_result, target_z);

    let flat_a = to_flat_set(&set_a, target_z);
    let flat_b = to_flat_set(&set_b, target_z);

    let expected: HashSet<_> = flat_a.difference(&flat_b).cloned().collect();

    assert_eq!(actual, expected, "Difference (A - B) should match");
}

#[test]
fn test_difference_three_sets() {
    let set_a = set_a();
    let set_b = set_b();
    let set_c = set_c();

    let diff_ab = set_a.difference(&set_b);
    let logic_result = diff_ab.difference(&set_c);

    let target_z = [
        set_a.max_z(),
        set_b.max_z(),
        set_c.max_z(),
        logic_result.max_z(),
    ]
    .into_iter()
    .max()
    .unwrap();

    let actual = to_flat_set(&logic_result, target_z);

    let flat_a = to_flat_set(&set_a, target_z);
    let flat_b = to_flat_set(&set_b, target_z);
    let flat_c = to_flat_set(&set_c, target_z);

    let diff_ab_set: HashSet<_> = flat_a.difference(&flat_b).cloned().collect();
    let expected: HashSet<_> = diff_ab_set.difference(&flat_c).cloned().collect();

    assert_eq!(actual, expected, "Difference ((A - B) - C) should match");
}
