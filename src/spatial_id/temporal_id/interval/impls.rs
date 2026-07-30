use core::fmt::Display;

use super::Interval;
use crate::error::Error;

impl Display for Interval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Interval::Whole => write!(f, "{}", Self::WHOLE_SECONDS),
            Interval::Day => write!(f, "{}", 60 * 60 * 24),
            Interval::Hour => write!(f, "{}", 60 * 60),
            Interval::Minute => write!(f, "{}", 60),
            Interval::Second => write!(f, "{}", 1),
        }
    }
}

impl Ord for Interval {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.seconds().cmp(&self.seconds())
    }
}

impl PartialOrd for Interval {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<u64> for Interval {
    type Error = Error;

    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        match seconds {
            Self::WHOLE_SECONDS => Ok(Interval::Whole),
            86400 => Ok(Interval::Day),
            3600 => Ok(Interval::Hour),
            60 => Ok(Interval::Minute),
            1 => Ok(Interval::Second),
            _ => Err(crate::SpatialIdError::TIntervalError { i: seconds }.into()),
        }
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
