use core::fmt::Display;

use super::Interval;
use crate::error::Error;

impl Display for Interval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Interval> for i64 {
    fn from(interval: Interval) -> i64 {
        interval.seconds() as i64
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;

    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        Interval::new(seconds)
    }
}

macro_rules! impl_try_from_unsigned {
    ($($t:ty),*) => {
        $(
            impl TryFrom<$t> for Interval {
                type Error = Error;

                fn try_from(seconds: $t) -> Result<Self, Self::Error> {
                    Self::try_from(seconds as u64)
                }
            }
        )*
    };
}

impl_try_from_unsigned!(u8, u16, u32, u128, usize, i8, i16, i32, i128, isize);
