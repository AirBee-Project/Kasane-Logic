#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::set::memory::SetOnMemory;
    use crate::spatial_id::collection::set::tests::{
        arb_small_set, set_a, set_b, set_c, to_flat_set,
    };
    use proptest::prelude::ProptestConfig;
    use proptest::proptest;
    use std::collections::HashSet;

    fn assert_union_consistency(
        logic_result: &SetOnMemory,
        inputs: &[&SetOnMemory],
        context_msg: &str,
    ) {
        let max_z = inputs
            .iter()
            .map(|s| s.max_z())
            .chain(std::iter::once(logic_result.max_z()))
            .max()
            .unwrap_or(0);

        let actual = to_flat_set(logic_result, max_z);

        let mut expected = HashSet::new();
        for set in inputs {
            let flat = to_flat_set(set, max_z);
            expected.extend(flat);
        }

        // 4. 比較
        assert_eq!(
            actual,
            expected,
            "{}\nInputs sizes: {:?}, Result size: {}",
            context_msg,
            inputs.iter().map(|s| s.size()).collect::<Vec<_>>(),
            logic_result.size()
        );
    }

    #[test]
    fn test_union() {
        let set_a = set_a();
        let set_b = set_b();

        let logic_result = set_a.union(&set_b);

        assert_union_consistency(
            &logic_result,
            &[&set_a, &set_b],
            "Manual union (A U B) failed",
        );
    }

    #[test]
    fn test_union_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let logic_union_ab = set_a.union(&set_b);
        let logic_result = logic_union_ab.union(&set_c);

        assert_union_consistency(
            &logic_result,
            &[&set_a, &set_b, &set_c],
            "Manual union (A U B U C) failed",
        );
    }

    /// 順番を入れ替えても計算結果が逆転しないことをテストする (可換性)
    #[test]
    fn test_union_commutative_manual() {
        let a = set_a();
        let c = set_c();

        let a_union_c = a.union(&c);
        let c_union_a = c.union(&a);

        let z = a_union_c.max_z().max(c_union_a.max_z());
        assert_eq!(to_flat_set(&a_union_c, z), to_flat_set(&c_union_a, z));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]
        #[test]
        fn random_test_union(
            set_a in arb_small_set(20),
            set_b in arb_small_set(20)
        ) {
            let logic_result = set_a.union(&set_b);

            assert_union_consistency(
                &logic_result,
                &[&set_a, &set_b],
                "Random union check failed"
            );
        }

        #[test]
        fn random_test_union_three_sets(
            set_a in arb_small_set(15),
            set_b in arb_small_set(15),
            set_c in arb_small_set(15)
        ) {
            let union_ab = set_a.union(&set_b);
            let logic_result = union_ab.union(&set_c);

            assert_union_consistency(
                &logic_result,
                &[&set_a, &set_b, &set_c],
                "Random 3-set union check failed"
            );
        }
    }
}
