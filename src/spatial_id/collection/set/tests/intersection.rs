#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::set::memory::SetOnMemory;
    use crate::spatial_id::collection::set::tests::{
        arb_small_set, set_a, set_b, set_c, to_flat_set,
    };
    use proptest::prelude::*;

    fn assert_intersection_consistency(
        logic_result: &SetOnMemory,
        inputs: &[&SetOnMemory],
        context_msg: &str,
    ) {
        if inputs.is_empty() {
            return;
        }

        let max_z = inputs
            .iter()
            .map(|s| s.max_z())
            .chain(std::iter::once(logic_result.max_z()))
            .max()
            .unwrap_or(0);

        let actual = to_flat_set(logic_result, max_z);

        let mut expected = to_flat_set(inputs[0], max_z);

        for other_set in &inputs[1..] {
            let other_flat = to_flat_set(other_set, max_z);
            expected.retain(|id| other_flat.contains(id));
        }

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
    fn test_intersection_two_sets() {
        futures::executor::block_on(async {
            let set_a = set_a().await;
            let set_b = set_b().await;

            let logic_result = set_a.intersection(&set_b).await;

            assert_intersection_consistency(
                &logic_result,
                &[&set_a, &set_b],
                "Manual intersection (A ∩ B) failed",
            );
        });
    }

    #[test]
    fn test_intersection_three_sets() {
        futures::executor::block_on(async {
            let set_a = set_a().await;
            let set_b = set_b().await;
            let set_c = set_c().await;

            // (A ∩ B) ∩ C
            let logic_inter_ab = set_a.intersection(&set_b).await;
            let logic_result = logic_inter_ab.intersection(&set_c).await;

            assert_intersection_consistency(
                &logic_result,
                &[&set_a, &set_b, &set_c],
                "Manual intersection (A ∩ B ∩ C) failed",
            );
        });
    }

    #[test]
    fn reverse() {
        futures::executor::block_on(async {
            // 結果が可換であることを確認する簡易テスト
            let set_a = set_a().await;
            let set_c = set_c().await;
            let mut a_and_c: Vec<_> = set_a.intersection(&set_c).await.single_ids().collect();
            let mut c_and_a: Vec<_> = set_c.intersection(&set_a).await.single_ids().collect();

            a_and_c.sort();
            c_and_a.sort();

            assert_eq!(a_and_c, c_and_a, "Intersection should be commutative")
        });
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn random_test_intersection(
            set_a in arb_small_set(20),
            set_b in arb_small_set(20)
        ) {
            futures::executor::block_on(async {
                let logic_result = set_a.intersection(&set_b).await;

                assert_intersection_consistency(
                    &logic_result,
                    &[&set_a, &set_b],
                    "Random intersection check failed"
                );
            });
        }

        #[test]
        fn random_test_intersection_three_sets(
            set_a in arb_small_set(15),
            set_b in arb_small_set(15),
            set_c in arb_small_set(15)
        ) {
            futures::executor::block_on(async {
                // (A ∩ B) ∩ C
                let inter_ab = set_a.intersection(&set_b).await;
                let logic_result = inter_ab.intersection(&set_c).await;

                assert_intersection_consistency(
                    &logic_result,
                    &[&set_a, &set_b, &set_c],
                    "Random 3-set intersection check failed"
                );
            });
        }
    }
}
