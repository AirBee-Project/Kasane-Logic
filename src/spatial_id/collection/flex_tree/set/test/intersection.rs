#[cfg(test)]
mod tests {
    use super::super::{arb_random_set_case, decompose_set_to_single_ids_at_zoom};
    use crate::{
        FlexTreeSet, SingleId, spatial_id::collection::flex_tree::set::test::sorted_single_ids,
    };
    use proptest::prelude::*;

    fn expected_intersection_single_ids(lhs: &FlexTreeSet, rhs: &FlexTreeSet) -> Vec<SingleId> {
        let common_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));
        let lhs_set = decompose_set_to_single_ids_at_zoom(lhs, common_z);
        let rhs_set = decompose_set_to_single_ids_at_zoom(rhs, common_z);

        let mut expected: Vec<SingleId> = lhs_set.intersection(&rhs_set).cloned().collect();
        expected.sort();
        expected
    }

    #[test]
    fn intersection_commutative_for_small_cases() {
        let mut lhs = FlexTreeSet::new();
        lhs.insert(SingleId::new(4, 3, 2, 1).unwrap());
        lhs.insert(SingleId::new(4, 3, 2, 2).unwrap());
        lhs.insert(SingleId::new(4, 4, 4, 4).unwrap());

        let mut rhs = FlexTreeSet::new();
        rhs.insert(SingleId::new(4, 3, 2, 2).unwrap());
        rhs.insert(SingleId::new(4, 4, 4, 4).unwrap());
        rhs.insert(SingleId::new(4, 5, 5, 5).unwrap());

        let lhs_rhs = &lhs & &rhs;
        let rhs_lhs = &rhs & &lhs;

        let target_z = lhs
            .max_zoomlevel()
            .unwrap_or(0)
            .max(rhs.max_zoomlevel().unwrap_or(0));
        assert_eq!(
            sorted_single_ids(&lhs_rhs, target_z),
            sorted_single_ids(&rhs_lhs, target_z)
        );
    }

    proptest! {
        #[test]
        fn intersection_matches_between_two_sets(
            lhs_case in arb_random_set_case(),
            rhs_case in arb_random_set_case(),
        ) {
            let lhs = lhs_case.build_set();
            let rhs = rhs_case.build_set();
            let common_z = lhs.max_zoomlevel().unwrap_or(0).max(rhs.max_zoomlevel().unwrap_or(0));

            let actual = &lhs & &rhs;
            let actual_single_ids = sorted_single_ids(&actual, common_z);
            let expected_single_ids = expected_intersection_single_ids(&lhs, &rhs);

            prop_assert_eq!(
                &actual_single_ids,
                &expected_single_ids,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );

            let reverse = &rhs & &lhs;
            prop_assert_eq!(
                &sorted_single_ids(&reverse, common_z),
                &expected_single_ids,
                "lhs={}\nrhs={}",
                lhs_case.debug_summary(),
                rhs_case.debug_summary(),
            );
        }
    }
}
