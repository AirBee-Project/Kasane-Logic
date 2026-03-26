#[cfg(test)]
mod tests {
    use crate::{
        SpatialIdSet, SpatialIds, VBitSet,
        spatial_id::collection::v_bit::set::test::{
            arb_small_set, set_a, set_b, set_c, to_flat_set,
        },
    };

    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Setを全探索して最大のズームレベル(Z)を算出するヘルパー
    fn calculate_max_z(set: &VBitSet) -> u8 {
        set.single_ids().map(|id| id.z()).max().unwrap_or(0)
    }

    /// 和集合の論理結果が、フラットに展開した数学的結果と一致するか検証する
    fn assert_union_consistency(logic_result: &VBitSet, inputs: &[&VBitSet], context_msg: &str) {
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

        // 和集合の期待値: 全ての入力Setのフラットな要素をガッチャンコ(extend)する
        let mut expected = HashSet::new();
        for &set in inputs {
            expected.extend(to_flat_set(set, max_z));
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
    fn test_union_two_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let logic_result = &set_a | &set_b;

        assert_union_consistency(
            &logic_result,
            &[&set_a, &set_b],
            "Manual union (A ∪ B) failed",
        );
    }

    #[test]
    fn test_union_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let logic_result = (&set_a | &set_b) | &set_c;

        assert_union_consistency(
            &logic_result,
            &[&set_a, &set_b, &set_c],
            "Manual union (A ∪ B ∪ C) failed",
        );
    }

    #[test]
    fn test_union_commutative_manual() {
        // 和集合演算が可換 (A ∪ C == C ∪ A) であることを確認
        let a_union_c = &set_a() | &set_c();
        let c_union_a = &set_c() | &set_a();

        let z = calculate_max_z(&a_union_c).max(calculate_max_z(&c_union_a));
        assert_eq!(
            to_flat_set(&a_union_c, z),
            to_flat_set(&c_union_a, z),
            "Union should be commutative"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn random_test_union(
            set_a in arb_small_set(20),
            set_b in arb_small_set(20)
        ) {
            let logic_result = &set_a | &set_b;

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
            let logic_result = (&set_a | &set_b) | &set_c;

            assert_union_consistency(
                &logic_result,
                &[&set_a, &set_b, &set_c],
                "Random 3-set union check failed"
            );
        }
    }
}
