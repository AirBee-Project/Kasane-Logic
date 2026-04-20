#[cfg(test)]
mod tests {
    use super::super::{arb_random_set_case, decompose_set_to_single_ids_at_zoom};
    use crate::{
        SpatilaIdSet, SingleId, spatial_id::collection::flex_tree::set::test::sorted_single_ids,
    };
    use proptest::prelude::*;

    fn expected_union_single_ids(lhs: &SpatilaIdSet, rhs: &SpatilaIdSet) -> Vec<SingleId> {
        let common_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));
        let mut union = decompose_set_to_single_ids_at_zoom(lhs, common_z);
        union.extend(decompose_set_to_single_ids_at_zoom(rhs, common_z));

        let mut expected: Vec<SingleId> = union.into_iter().collect();
        expected.sort();
        expected
    }

    fn expected_union_of_three_single_ids(
        a: &SpatilaIdSet,
        b: &SpatilaIdSet,
        c: &SpatilaIdSet,
    ) -> Vec<SingleId> {
        let common_z = a
            .max_zoomlevel()
            .unwrap_or(0)
            .max(b.max_zoomlevel().unwrap_or(0))
            .max(c.max_zoomlevel().unwrap_or(0));

        let mut union = decompose_set_to_single_ids_at_zoom(a, common_z);
        union.extend(decompose_set_to_single_ids_at_zoom(b, common_z));
        union.extend(decompose_set_to_single_ids_at_zoom(c, common_z));

        let mut expected: Vec<SingleId> = union.into_iter().collect();
        expected.sort();
        expected
    }

    /// 2つの Set の和集合演算が交換法則（A∪B = B∪A）を満たすことを固定ケースで検証する。
    #[test]
    fn union_commutative_for_small_cases() {
        let mut lhs = SpatilaIdSet::new();
        lhs.insert(SingleId::new(4, 3, 2, 1).unwrap());
        lhs.insert(SingleId::new(4, 3, 2, 2).unwrap());

        let mut rhs = SpatilaIdSet::new();
        rhs.insert(SingleId::new(4, 3, 2, 2).unwrap());
        rhs.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let lhs_rhs = &lhs | &rhs;
        let rhs_lhs = &rhs | &lhs;

        let target_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));
        assert_eq!(
            sorted_single_ids(&lhs_rhs, target_z),
            sorted_single_ids(&rhs_lhs, target_z)
        );
    }

    /// 3つの Set の和集合演算結果が手計算した期待値と一致することを固定ケースで検証する。
    #[test]
    fn union_of_three_sets_matches_expected() {
        let mut a = SpatilaIdSet::new();
        a.insert(SingleId::new(4, 3, 2, 1).unwrap());
        a.insert(SingleId::new(4, 3, 2, 2).unwrap());

        let mut b = SpatilaIdSet::new();
        b.insert(SingleId::new(4, 3, 2, 2).unwrap());
        b.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let mut c = SpatilaIdSet::new();
        c.insert(SingleId::new(4, 5, 5, 5).unwrap());

        let ab = &a | &b;
        let abc = &ab | &c;

        let target_z = a
            .max_zoomlevel()
            .unwrap_or(0)
            .max(b.max_zoomlevel().unwrap_or(0))
            .max(c.max_zoomlevel().unwrap_or(0));

        let expected = vec![
            SingleId::new(4, 3, 2, 1).unwrap(),
            SingleId::new(4, 3, 2, 2).unwrap(),
            SingleId::new(4, 4, 4, 4).unwrap(),
            SingleId::new(4, 5, 5, 5).unwrap(),
        ];
        assert_eq!(sorted_single_ids(&abc, target_z), expected);
    }

    proptest! {
        /// 2つの Set の和集合演算結果が共通ズームへ正規化した単一セル集合の和集合と一致することを検証する。
        #[test]
        fn union_matches_between_two_sets(
            lhs_case in arb_random_set_case(),
            rhs_case in arb_random_set_case(),
        ) {
            let lhs = lhs_case.build_set();
            let rhs = rhs_case.build_set();
            let common_z = lhs.max_zoomlevel().unwrap_or(0).max(rhs.max_zoomlevel().unwrap_or(0));

            let actual = &lhs | &rhs;
            let actual_single_ids = sorted_single_ids(&actual, common_z);
            let expected_single_ids = expected_union_single_ids(&lhs, &rhs);

            prop_assert_eq!(
                &actual_single_ids,
                &expected_single_ids,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );

            let reverse = &rhs | &lhs;
            prop_assert_eq!(
                &sorted_single_ids(&reverse, common_z),
                &expected_single_ids,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );
        }

        /// 3つの Set の和集合演算結果が期待値と一致し、かつ結合法則（(A∪B)∪C = A∪(B∪C)）を満たすことを検証する。
        #[test]
        fn union_matches_between_three_sets(
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

            let ab = &a | &b;
            let actual = &ab | &c;
            let actual_single_ids = sorted_single_ids(&actual, common_z);
            let expected_single_ids = expected_union_of_three_single_ids(&a, &b, &c);

            prop_assert_eq!(
                &actual_single_ids,
                &expected_single_ids,
                "a={}\nb={}\nc={}",
                a_case.debug_summary(),
                b_case.debug_summary(),
                c_case.debug_summary(),
            );

            let bc = &b | &c;
            let actual_associative = &a | &bc;
            prop_assert_eq!(
                &sorted_single_ids(&actual_associative, common_z),
                &expected_single_ids,
                "a={}\nb={}\nc={}",
                a_case.debug_summary(),
                b_case.debug_summary(),
                c_case.debug_summary(),
            );
        }
    }
}
