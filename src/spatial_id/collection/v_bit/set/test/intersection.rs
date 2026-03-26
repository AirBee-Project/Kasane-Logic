#[cfg(test)]
mod tests {
    use crate::{
        SpatialIdSet, SpatialIds, VBitSet,
        spatial_id::collection::v_bit::set::test::{
            arb_small_set, set_a, set_b, set_c, to_flat_set,
        },
    };

    use proptest::prelude::*;

    /// VBitSetを全探索して最大のズームレベル(Z)を算出するヘルパー
    fn calculate_max_z(set: &VBitSet) -> u8 {
        set.single_ids().map(|id| id.z()).max().unwrap_or(0)
    }

    /// 交差判定の論理結果が、フラットに展開した数学的結果と一致するか検証する
    fn assert_intersection_consistency(
        logic_result: &VBitSet,
        inputs: &[&VBitSet],
        context_msg: &str,
    ) {
        if inputs.is_empty() {
            return;
        }

        // 入力Setと出力Setすべての single_ids() を走査して最大のZを求める
        let max_z = inputs
            .iter()
            .map(|&s| calculate_max_z(s))
            .chain(std::iter::once(calculate_max_z(logic_result)))
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
        let set_a = set_a();
        let set_b = set_b();
        let logic_result = set_a.clone() & set_b.clone();

        assert_intersection_consistency(
            &logic_result,
            &[&set_a, &set_b],
            "Manual intersection (A ∩ B) failed",
        );
    }

    #[test]
    fn test_intersection_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let logic_inter_ab = set_a.clone() & set_b.clone();
        let logic_result = logic_inter_ab & set_c.clone();

        assert_intersection_consistency(
            &logic_result,
            &[&set_a, &set_b, &set_c],
            "Manual intersection (A ∩ B ∩ C) failed",
        );
    }

    #[test]
    fn test_intersection_commutativity() {
        // 交差演算が可換(A ∩ C == C ∩ A)であることを確認
        let mut a_and_c: Vec<_> = (set_a() & set_c()).single_ids().collect();
        let mut c_and_a: Vec<_> = (set_c() & set_a()).single_ids().collect();

        a_and_c.sort();
        c_and_a.sort();

        assert_eq!(a_and_c, c_and_a, "Intersection should be commutative");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn random_test_intersection(
            set_a in arb_small_set(20),
            set_b in arb_small_set(20)
        ) {
            let logic_result = set_a.clone() & set_b.clone();
            assert_intersection_consistency(
                &logic_result,
                &[&set_a, &set_b],
                "Random intersection check failed"
            );
        }

        #[test]
        fn random_test_intersection_three_sets(
            set_a in arb_small_set(15),
            set_b in arb_small_set(15),
            set_c in arb_small_set(15)
        ) {
            let inter_ab = set_a.clone() & set_b.clone();
            let logic_result = inter_ab & set_c.clone();

            assert_intersection_consistency(
                &logic_result,
                &[&set_a, &set_b, &set_c],
                "Random 3-set intersection check failed"
            );
        }
    }
}
