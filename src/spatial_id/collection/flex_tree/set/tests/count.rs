#[cfg(test)]
mod tests {
    use super::super::{RandomSetInsert, arb_random_set_case};
    use crate::SpatialIdSet;
    use proptest::prelude::*;

    fn assert_count_consistent(set: &SpatialIdSet) {
        assert_eq!(set.count(), set.iter().count());
    }

    /// O(1) 化前の `max_zoomlevel` と同じ全探索ロジックで最大ズームを求める参照実装。
    fn brute_force_max_zoomlevel(set: &SpatialIdSet) -> Option<u8> {
        set.iter()
            .map(|id| id.f_zoomlevel().max(id.x_zoomlevel()).max(id.y_zoomlevel()))
            .max()
    }

    fn assert_max_zoomlevel_consistent(set: &SpatialIdSet) {
        assert_eq!(set.max_zoomlevel(), brute_force_max_zoomlevel(set));
    }

    fn remove_insert_pattern(set: &mut SpatialIdSet, insert: &RandomSetInsert) {
        match insert {
            RandomSetInsert::Single(single_id) => {
                let _ = set.remove(single_id.clone());
            }
            RandomSetInsert::Range(range_id) => {
                let _ = set.remove(range_id.clone());
            }
        }
    }

    proptest! {
        /// ランダム挿入後の Set で count() と実際の要素数が一致することを検証する。
        #[ignore]
        #[test]
        fn count_matches_iter_count_after_random_insert(case in arb_random_set_case()) {
            let set = case.build_set();
            prop_assert_eq!(set.count(), set.iter().count(), "{}", case.debug_summary());
        }

        /// ランダム挿入後、キャッシュした max_zoomlevel が全探索の参照実装と一致することを検証する。
        #[ignore]
        #[test]
        fn max_zoomlevel_matches_brute_force_after_random_insert(case in arb_random_set_case()) {
            let set = case.build_set();
            prop_assert_eq!(
                set.max_zoomlevel(),
                brute_force_max_zoomlevel(&set),
                "{}",
                case.debug_summary()
            );
        }

        /// ランダム挿入と削除を繰り返しても max_zoomlevel が全探索の参照実装と一致することを検証する。
        #[ignore]
        #[test]
        fn max_zoomlevel_matches_brute_force_after_random_insert_and_remove(
            base_case in arb_random_set_case(),
            remove_case in arb_random_set_case(),
        ) {
            let mut set = base_case.build_set();
            for insert in &remove_case.inserts {
                remove_insert_pattern(&mut set, insert);
                prop_assert_eq!(
                    set.max_zoomlevel(),
                    brute_force_max_zoomlevel(&set),
                    "base={}\nremove={}",
                    base_case.debug_summary(),
                    remove_case.debug_summary(),
                );
            }
        }

        /// ランダム挿入とランダム削除を繰り返した後でも count() と実際の要素数が一致することを検証する。
        #[ignore]
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
