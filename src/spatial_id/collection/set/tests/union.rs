#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::spatial_id::collection::set::tests::{set_a, set_b, set_c, to_flat_set};

    #[test]
    fn test_union_consistency() {
        let set_a = set_a();
        let set_b = set_b();

        let logic_result = set_a.union(&set_b);

        let target_z = [set_a.max_z(), set_b.max_z(), logic_result.max_z()]
            .into_iter()
            .max()
            .unwrap();

        let actual = to_flat_set(&logic_result, target_z);

        let flat_a = to_flat_set(&set_a, target_z);
        let flat_b = to_flat_set(&set_b, target_z);

        let expected: HashSet<_> = flat_a.union(&flat_b).cloned().collect();

        assert_eq!(
            actual, expected,
            "SetLogic::union result should match HashSet::union"
        );
    }

    #[test]
    fn test_union_three_sets() {
        let set_a = set_a();
        let set_b = set_b();
        let set_c = set_c();

        let logic_union_ab = set_a.union(&set_b);
        let logic_result = logic_union_ab.union(&set_c);

        let target_z = [
            set_a.max_z(),
            set_b.max_z(),
            set_c.max_z(),
            logic_result.max_z(),
        ]
        .into_iter()
        .max()
        .unwrap();

        let actual = to_flat_set(&logic_result, target_z);

        let flat_a = to_flat_set(&set_a, target_z);
        let flat_b = to_flat_set(&set_b, target_z);
        let flat_c = to_flat_set(&set_c, target_z);

        let mut expected = flat_a;
        expected.extend(flat_b); // A ∪ B
        expected.extend(flat_c); // (A ∪ B) ∪ C

        assert_eq!(actual, expected, "Union of 3 sets (A, B, C) should match");
    }

    ///順番を入れ替えても計算結果が逆転しないことをテストする
    #[test]
    fn reverse() {
        let mut a_and_c: Vec<_> = set_a().union(&set_c()).single_ids().collect();
        let mut c_and_a: Vec<_> = set_c().union(&set_a()).single_ids().collect();

        a_and_c.sort();
        c_and_a.sort();

        assert_eq!(a_and_c, c_and_a)
    }
}
