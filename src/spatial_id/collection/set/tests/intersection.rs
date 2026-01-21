#[cfg(test)]
mod tests {
    use crate::{
        SingleId,
        spatial_id::collection::set::tests::{set_a, set_b, set_c},
    };

    ///交わりがない場合のテスト
    #[test]
    fn no_intersection() {
        let a_and_b = set_a().intersection(&set_b());
        assert_eq!(a_and_b.is_empty(), true)
    }

    ///交わりがある場合のテスト
    #[test]
    fn normal_intersection_a_and_c() {
        let a_and_c = set_a().intersection(&set_c());

        //交わる
        assert_eq!(a_and_c.is_empty(), false);

        //answer
        let mut answer = vec![
            SingleId::new(3, 2, 2, 2).unwrap(),
            SingleId::new(3, 2, 3, 2).unwrap(),
        ];

        let mut result: Vec<_> = a_and_c.single_ids().collect();

        answer.sort();
        result.sort();

        assert_eq!(answer, result)
    }

    ///交わりがある場合のテスト
    #[test]
    fn normal_intersection_b_and_c() {
        let b_and_c = set_b().intersection(&set_c());

        //交わる
        assert_eq!(b_and_c.is_empty(), false);

        //answer
        let mut answer = vec![SingleId::new(3, 4, 4, 4).unwrap()];

        let mut result: Vec<_> = b_and_c.single_ids().collect();

        answer.sort();
        result.sort();

        assert_eq!(answer, result)
    }

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
