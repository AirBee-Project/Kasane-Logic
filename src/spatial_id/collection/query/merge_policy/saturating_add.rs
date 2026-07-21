pub trait Add: Sized {
    fn saturating_add(self, rhs: Self) -> Self;
}

macro_rules! impl_saturating_add_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl Add for $t {
                fn saturating_add(self, rhs: Self) -> Self {
                    <$t>::saturating_add(self, rhs)
                }
            }
        )*
    };
}
impl_saturating_add_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! impl_saturating_add_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl Add for $t {
                fn saturating_add(self, rhs: Self) -> Self {
                    self + rhs
                }
            }
        )*
    };
}
impl_saturating_add_float!(f32, f64);
