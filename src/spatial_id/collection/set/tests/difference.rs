#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::set::memory::SetOnMemory;
    use crate::spatial_id::collection::set::tests::{
        arb_small_set, set_a, set_b, set_c, to_flat_set,
    };
    use proptest::prelude::*;

    fn assert_difference_consistency(
        logic_result: &SetOnMemory,
        initial_set: &SetOnMemory,
        subtractors: &[&SetOnMemory],
        context_msg: &str,
    ) {
        let max_z = std::iter::once(logic_result.max_z())
            .chain(std::iter::once(initial_set.max_z()))
            .chain(subtractors.iter().map(|s| s.max_z()))
            .max()
            .unwrap_or(0);

        let actual = to_flat_set(logic_result, max_z);

        let mut expected = to_flat_set(initial_set, max_z);

        for sub_set in subtractors {
            let sub_flat = to_flat_set(sub_set, max_z);
            // sub_flat に含まれているものは削除する (A - B)
            expected.retain(|id| !sub_flat.contains(id));
        }

        assert_eq!(
            actual,
            expected,
            "{}\nInitial size: {}, Subtractors sizes: {:?}, Result size: {}",
            context_msg,
            initial_set.size(),
            subtractors.iter().map(|s| s.size()).collect::<Vec<_>>(),
            logic_result.size()
        );
    }

    #[test]
    fn test_difference_two_sets() {
        let set_a = set_a();
        let set_b = set_b();

        let logic_result = set_a.difference(&set_b);

        assert_difference_consistency(
            &logic_result,
            &set_a,
            &[&set_b],
            "Manual difference (A - B) failed",
        );
    }

    #[test]
    fn test_difference_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let diff_ab = set_a.difference(&set_b);
        let logic_result = diff_ab.difference(&set_c);

        assert_difference_consistency(
            &logic_result,
            &set_a,
            &[&set_b, &set_c],
            "Manual difference ((A - B) - C) failed",
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn random_test_difference(
            set_a in arb_small_set(20),
            set_b in arb_small_set(20)
        ) {
            let logic_result = set_a.difference(&set_b);

            assert_difference_consistency(
                &logic_result,
                &set_a,
                &[&set_b],
                "Random difference check failed"
            );
        }

        #[test]
        fn random_test_difference_three_sets(
            set_a in arb_small_set(15),
            set_b in arb_small_set(15),
            set_c in arb_small_set(15)
        ) {
            // (A - B) - C
            let diff_ab = set_a.difference(&set_b);
            let logic_result = diff_ab.difference(&set_c);

            assert_difference_consistency(
                &logic_result,
                &set_a,
                &[&set_b, &set_c],
                "Random 3-set difference check failed"
            );
        }

        #[test]
        fn random_test_self_difference(
            set_a in arb_small_set(20)
        ) {
            let logic_result = set_a.difference(&set_a);

            // 結果は空であるはず
            prop_assert!(logic_result.is_empty(),
                "A - A should be empty. Result size: {}", logic_result.size());
        }
    }
}
