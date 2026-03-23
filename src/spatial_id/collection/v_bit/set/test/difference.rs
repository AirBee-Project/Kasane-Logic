#[cfg(test)]
mod tests {
    use crate::{
        SpatialIdSet, SpatialIds, VBitSet,
        spatial_id::collection::v_bit::set::test::{
            arb_small_set, set_a, set_b, set_c, to_flat_set,
        },
    };

    use proptest::prelude::*;

    /// Setを全探索して最大のズームレベル(Z)を算出するヘルパー
    fn calculate_max_z(set: &VBitSet) -> u8 {
        set.single_ids().map(|id| id.z()).max().unwrap_or(0)
    }

    /// 差集合の論理結果が、フラットに展開した数学的結果と一致するか検証する
    fn assert_difference_consistency(
        logic_result: &VBitSet,
        initial_set: &VBitSet,
        subtractors: &[&VBitSet],
        context_msg: &str,
    ) {
        // 全ての入力・出力から最大のZを求める
        let max_z = subtractors
            .iter()
            .map(|&s| calculate_max_z(s))
            .chain(std::iter::once(calculate_max_z(initial_set)))
            .chain(std::iter::once(calculate_max_z(logic_result)))
            .max()
            .unwrap_or(0);

        let actual = to_flat_set(logic_result, max_z);
        let mut expected = to_flat_set(initial_set, max_z);

        for &sub_set in subtractors {
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

        let logic_result = &set_a - &set_b; // 演算子オーバーロードを利用

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

        let logic_result = (&set_a - &set_b) - &set_c;

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
            let logic_result = &set_a - &set_b;

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
            let logic_result = (&set_a - &set_b) - &set_c;

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
            let logic_result = &set_a - &set_a;

            // 自身から自身を引いた結果は必ず空になるはず
            prop_assert!(logic_result.is_empty(),
                "A - A should be empty. Result size: {}", logic_result.size());
        }
    }
}
