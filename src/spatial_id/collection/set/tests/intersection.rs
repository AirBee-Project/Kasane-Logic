#[cfg(test)]
mod tests {
    #[test]
    fn no_intersection() {
        use crate::spatial_id::collection::set::tests::{set_a, set_b};

        for ele in set_a().flex_ids() {
            println!("{:?},", ele);
        }

        let a_and_b = set_a().intersection(&set_b());

        assert_eq!(a_and_b.is_empty(), false)
    }

    #[test]
    fn normal_intersection() {}
}
