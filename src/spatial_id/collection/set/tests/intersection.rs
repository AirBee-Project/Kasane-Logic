#[cfg(test)]
mod tests {
    use crate::{
        SingleId,
        spatial_id::collection::set::tests::{set_a, set_b, set_c},
    };

    #[test]
    fn no_intersection() {
        let a_and_b = set_a().intersection(&set_b());
        assert_eq!(a_and_b.is_empty(), true)
    }

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
}
