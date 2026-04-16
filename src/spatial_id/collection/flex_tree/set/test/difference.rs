#[cfg(test)]
mod tests {
    use super::super::{arb_random_set_case, decompose_set_to_single_ids_at_zoom};
    use crate::{
        FlexTreeSet, SingleId, spatial_id::collection::flex_tree::set::test::sorted_single_ids,
    };
    use proptest::prelude::*;

    fn expected_difference_single_ids(lhs: &FlexTreeSet, rhs: &FlexTreeSet) -> Vec<SingleId> {
        let common_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));
        let lhs_set = decompose_set_to_single_ids_at_zoom(lhs, common_z);
        let rhs_set = decompose_set_to_single_ids_at_zoom(rhs, common_z);

        let mut expected: Vec<SingleId> = lhs_set.difference(&rhs_set).cloned().collect();
        expected.sort();
        expected
    }

    fn expected_difference_of_three_single_ids(
        a: &FlexTreeSet,
        b: &FlexTreeSet,
        c: &FlexTreeSet,
    ) -> Vec<SingleId> {
        let common_z = a
            .max_zoomlevel()
            .unwrap_or(0)
            .max(b.max_zoomlevel().unwrap_or(0))
            .max(c.max_zoomlevel().unwrap_or(0));

        let mut removed = decompose_set_to_single_ids_at_zoom(b, common_z);
        removed.extend(decompose_set_to_single_ids_at_zoom(c, common_z));

        let a_set = decompose_set_to_single_ids_at_zoom(a, common_z);
        let mut expected: Vec<SingleId> = a_set.difference(&removed).cloned().collect();
        expected.sort();
        expected
    }

    /// 差集合演算が非可換であり、A-B と B-A がそれぞれ期待値と一致することを固定ケースで検証する。
    #[test]
    fn difference_non_commutative_for_small_cases() {
        let mut lhs = FlexTreeSet::new();
        lhs.insert(SingleId::new(4, 3, 2, 1).unwrap());
        lhs.insert(SingleId::new(4, 3, 2, 2).unwrap());

        let mut rhs = FlexTreeSet::new();
        rhs.insert(SingleId::new(4, 3, 2, 2).unwrap());
        rhs.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let lhs_minus_rhs = &lhs - &rhs;
        let rhs_minus_lhs = &rhs - &lhs;

        let target_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));

        assert_eq!(
            sorted_single_ids(&lhs_minus_rhs, target_z),
            vec![SingleId::new(4, 3, 2, 1).unwrap()]
        );
        assert_eq!(
            sorted_single_ids(&rhs_minus_lhs, target_z),
            vec![SingleId::new(4, 4, 4, 4).unwrap()]
        );
        assert!(lhs_minus_rhs != rhs_minus_lhs);
    }

    /// 3つの Set で (A-B)-C が A-(B∪C) の期待値と一致することを固定ケースで検証する。
    #[test]
    fn difference_of_three_sets_matches_expected() {
        let mut a = FlexTreeSet::new();
        a.insert(SingleId::new(4, 3, 2, 1).unwrap());
        a.insert(SingleId::new(4, 3, 2, 2).unwrap());
        a.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let mut b = FlexTreeSet::new();
        b.insert(SingleId::new(4, 3, 2, 2).unwrap());

        let mut c = FlexTreeSet::new();
        c.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let ab = &a - &b;
        let actual = &ab - &c;

        let target_z = a
            .max_zoomlevel()
            .unwrap_or(0)
            .max(b.max_zoomlevel().unwrap_or(0))
            .max(c.max_zoomlevel().unwrap_or(0));

        let expected = vec![SingleId::new(4, 3, 2, 1).unwrap()];
        assert_eq!(sorted_single_ids(&actual, target_z), expected);
    }

    proptest! {
        /// 2つの Set の差集合演算結果が共通ズームへ正規化した単一セル集合の差集合と一致することを検証する。
        #[test]
        fn difference_matches_between_two_sets(
            lhs_case in arb_random_set_case(),
            rhs_case in arb_random_set_case(),
        ) {
            let lhs = lhs_case.build_set();
            let rhs = rhs_case.build_set();
            let common_z = lhs.max_zoomlevel().unwrap_or(0).max(rhs.max_zoomlevel().unwrap_or(0));

            let actual = &lhs - &rhs;
            let actual_single_ids = sorted_single_ids(&actual, common_z);
            let expected_single_ids = expected_difference_single_ids(&lhs, &rhs);

            prop_assert_eq!(
                &actual_single_ids,
                &expected_single_ids,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );

            let reverse = &rhs - &lhs;
            let expected_reverse = expected_difference_single_ids(&rhs, &lhs);
            prop_assert_eq!(
                &sorted_single_ids(&reverse, common_z),
                &expected_reverse,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );
        }

        /// 3つの Set の差集合演算結果が期待値と一致し、かつ (A-B)-C = A-(B∪C) を満たすことを検証する。
        #[test]
        fn difference_matches_between_three_sets(
            a_case in arb_random_set_case(),
            b_case in arb_random_set_case(),
            c_case in arb_random_set_case(),
        ) {
            let a = a_case.build_set();
            let b = b_case.build_set();
            let c = c_case.build_set();

            let common_z = a
                .max_zoomlevel()
                .unwrap_or(0)
                .max(b.max_zoomlevel().unwrap_or(0))
                .max(c.max_zoomlevel().unwrap_or(0));

            let ab = &a - &b;
            let actual = &ab - &c;
            let actual_single_ids = sorted_single_ids(&actual, common_z);
            let expected_single_ids = expected_difference_of_three_single_ids(&a, &b, &c);

            prop_assert_eq!(
                &actual_single_ids,
                &expected_single_ids,
                "a={}\nb={}\nc={}",
                a_case.debug_summary(),
                b_case.debug_summary(),
                c_case.debug_summary(),
            );

            let bc_union = &b | &c;
            let actual_set_algebra = &a - &bc_union;
            prop_assert_eq!(
                &sorted_single_ids(&actual_set_algebra, common_z),
                &expected_single_ids,
                "a={}\nb={}\nc={}",
                a_case.debug_summary(),
                b_case.debug_summary(),
                c_case.debug_summary(),
            );
        }
    }
}
