#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        SetOnMemory, SingleId,
        spatial_id::collection::set::tests::{set_a, set_c},
    };

    #[test]
    fn normal_intersection_a_and_c() {}

    ///順番を入れ替えても計算結果が逆転しないことをテストする
    #[test]
    fn reverse() {
        let mut a_and_c: Vec<_> = set_a().intersection(&set_c()).single_ids().collect();
        let mut c_and_a: Vec<_> = set_c().intersection(&set_a()).single_ids().collect();

        a_and_c.sort();
        c_and_a.sort();

        assert_eq!(a_and_c, c_and_a)
    }
}
