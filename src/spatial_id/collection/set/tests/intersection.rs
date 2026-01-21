#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::spatial_id::collection::set::tests::{set_a, set_b, set_c, to_flat_set};

    #[test]
    fn test_intersection_two_sets() {
        let set_a = set_a();
        let set_b = set_b();

        let logic_result = set_a.intersection(&set_b);

        let target_z = [set_a.max_z(), set_b.max_z(), logic_result.max_z()]
            .into_iter()
            .max()
            .unwrap();

        let actual = to_flat_set(&logic_result, target_z);

        let flat_a = to_flat_set(&set_a, target_z);
        let flat_b = to_flat_set(&set_b, target_z);

        let expected: HashSet<_> = flat_a.intersection(&flat_b).cloned().collect();

        assert_eq!(actual, expected, "Intersection of A and B should match");
    }

    #[test]
    fn test_intersection_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let logic_inter_ab = set_a.intersection(&set_b);
        let logic_result = logic_inter_ab.intersection(&set_c);

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

        // A ∩ B
        let inter_ab: HashSet<_> = flat_a.intersection(&flat_b).cloned().collect();
        // (A ∩ B) ∩ C
        let expected: HashSet<_> = inter_ab.intersection(&flat_c).cloned().collect();

        assert_eq!(
            actual, expected,
            "Intersection of 3 sets (A, B, C) should match"
        );
    }

    #[test]
    fn reverse() {
        let mut a_and_c: Vec<_> = set_a().intersection(&set_c()).single_ids().collect();
        let mut c_and_a: Vec<_> = set_c().intersection(&set_a()).single_ids().collect();

        a_and_c.sort();
        c_and_a.sort();

        assert_eq!(a_and_c, c_and_a)
    }
}
