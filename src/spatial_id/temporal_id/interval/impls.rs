use super::Interval;
use crate::{SpatialIdError, error::Error};

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

    #[allow(clippy::manual_is_multiple_of)]
    fn try_from(seconds: u64) -> Result<Self, Self::Error> {
        match seconds {
            Self::WHOLE_SECONDS => Ok(Interval::Whole),
            86400 => Ok(Interval::Day),
            3600 => Ok(Interval::Hour),
            60 => Ok(Interval::Minute),
            1 => Ok(Interval::Second),
            _ => {
                if seconds > 86400 && seconds % 86400 == 0 {
                    let m = seconds / 86400;
                    if m.is_power_of_two() {
                        let k = m.trailing_zeros() as u8;
                        if k < Self::WHOLE_POW {
                            return Ok(Interval::DayPow { k });
                        }
                    }
                }
                Err(SpatialIdError::TIntervalError { i: seconds }.into())
            }
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
