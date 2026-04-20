#[cfg(test)]
mod tests {
    use super::super::{RandomSetInsert, arb_random_set_case};
    use crate::{SpatilaIdSet, RangeId, SingleId};
    use proptest::prelude::*;

    fn assert_count_consistent(set: &SpatilaIdSet) {
        assert_eq!(set.count(), set.iter().count());
    }

    fn remove_insert_pattern(set: &mut SpatilaIdSet, insert: &RandomSetInsert) {
        match insert {
            RandomSetInsert::Single(single_id) => {
                let _ = set.remove(single_id).collect::<Vec<_>>();
            }
            RandomSetInsert::Range(range_id) => {
                let _ = set.remove(range_id).collect::<Vec<_>>();
            }
        }
    }

    /// 挿入後の Set で count() と実際の要素数が一致することを固定ケースで検証する。
    #[test]
    fn count_matches_iter_count_fixed_insert_case() {
        let mut set = SpatilaIdSet::new();
        set.insert(SingleId::new(4, 3, 2, 1).unwrap());
        set.insert(SingleId::new(4, 3, 2, 2).unwrap());
        set.insert(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap());

        assert_count_consistent(&set);
    }

    /// 削除後の Set で count() と実際の要素数が一致することを固定ケースで検証する。
    #[test]
    fn count_matches_iter_count_fixed_remove_case() {
        let mut set = SpatilaIdSet::new();
        let remove_target = SingleId::new(4, 3, 2, 2).unwrap();

        set.insert(SingleId::new(4, 3, 2, 1).unwrap());
        set.insert(remove_target.clone());
        set.insert(RangeId::new(3, [0, 1], [1, 2], [3, 3]).unwrap());
        assert_count_consistent(&set);

        let _ = set.remove(&remove_target).collect::<Vec<_>>();
        assert_count_consistent(&set);
    }

    proptest! {
        /// ランダム挿入後の Set で count() と実際の要素数が一致することを検証する。
        #[test]
        fn count_matches_iter_count_after_random_insert(case in arb_random_set_case()) {
            let set = case.build_set();
            prop_assert_eq!(set.count(), set.iter().count(), "{}", case.debug_summary());
        }

        /// ランダム挿入とランダム削除を繰り返した後でも count() と実際の要素数が一致することを検証する。
        #[test]
        fn count_matches_iter_count_after_random_insert_and_remove(
            base_case in arb_random_set_case(),
            remove_case in arb_random_set_case(),
        ) {
            let mut set = base_case.build_set();
            for insert in &remove_case.inserts {
                remove_insert_pattern(&mut set, insert);
                prop_assert_eq!(
                    set.count(),
                    set.iter().count(),
                    "base={}\nremove={}",
                    base_case.debug_summary(),
                    remove_case.debug_summary(),
                );
            }
        }
    }
}
