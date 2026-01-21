#[cfg(test)]
mod tests {
    use crate::spatial_id::collection::set::tests::{set_a, set_b, set_c};

    #[test]
    fn not_equal_a_b() {
        assert_eq!(set_a().equal(&set_b()), false);
    }

    #[test]
    fn not_equal_a_c() {
        assert_eq!(set_a().equal(&set_c()), false);
    }

    #[test]
    fn not_equal_b_c() {
        assert_eq!(set_b().equal(&set_c()), false);
    }

    #[test]
    fn equal_a() {
        assert_eq!(set_a().equal(&set_a()), true);
    }

    #[test]
    fn equal_b() {
        assert_eq!(set_b().equal(&set_b()), true);
    }

    #[test]
    fn equal_c() {
        assert_eq!(set_c().equal(&set_c()), true);
    }
}
