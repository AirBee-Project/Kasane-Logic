#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::set::tests::{set_a, set_b, set_c};
    use std::collections::HashSet;

    ///交わりがない場合のテスト
    #[test]
    fn no_intersection() {
        let mut a_union_b: Vec<_> = set_a().union(&set_b()).single_ids().collect();

        let mut a_single_id: Vec<_> = set_a().single_ids().collect();
        let b_single_id: Vec<_> = set_b().single_ids().collect();

        a_single_id.extend(b_single_id);

        a_single_id.sort();
        a_union_b.sort();

        assert_eq!(a_single_id, a_union_b)
    }

    ///交わりがある場合のテスト
    #[test]
    fn normal_intersection_a_union_c() {
        let a_union_c: Vec<_> = set_a().union(&set_c()).single_ids().collect();
        let mut set: HashSet<_> = a_union_c.iter().collect();

        //重複がないことを確認
        assert_eq!(set.len(), a_union_c.len());

        //範囲があっていることを確認
        for single_id in set_a().single_ids() {
            set.remove(&single_id);
        }

        for single_id in set_c().single_ids() {
            set.remove(&single_id);
        }

        assert_eq!(true, set.is_empty())
    }

    ///交わりがある場合のテスト
    #[test]
    fn normal_intersection_b_union_c() {
        let a_union_c: Vec<_> = set_b().union(&set_c()).single_ids().collect();
        let mut set: HashSet<_> = a_union_c.iter().collect();

        //重複がないことを確認
        assert_eq!(set.len(), a_union_c.len());

        //範囲があっていることを確認
        for single_id in set_b().single_ids() {
            set.remove(&single_id);
        }

        for single_id in set_c().single_ids() {
            set.remove(&single_id);
        }

        assert_eq!(true, set.is_empty())
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
