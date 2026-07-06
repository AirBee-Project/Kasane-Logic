use super::Interval;
use crate::{SpatialIdError, error::Error};

impl Interval {
    /// このライブラリが扱える全時間の秒数。86400 × 2^47`（約3,850億年）。
    pub const WHOLE_SECONDS: u64 = 86400 << 47;

    /// 最も粗い時間区間を表す二進層の指数。
    pub const WHOLE_POW: u8 = 47;

    /// 秒数から[Interval]型を作成する。
    ///
    /// 約数鎖に無い秒数は
    /// [`SpatialIdError::TIntervalError`] を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::Hour);
    /// assert_eq!(Interval::new(86400 * 4).unwrap(), Interval::day_pow(2).unwrap());
    /// assert!(Interval::new(7200).is_err()); // 約数鎖に無い
    /// ```
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        Interval::try_from(seconds)
    }

    /// 二進層の間隔 `Day·2^k` を作成する。
    ///
    /// `k = 0` は [`Day`](Self::Day)、`k = 47` は [`Whole`](Self::Whole) に一致する。
    /// `k > 47` は[`SpatialIdError::TDayPowOutOfRange`]を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// let two_days = Interval::day_pow(1).unwrap();
    /// assert_eq!(two_days.seconds(), 86400 * 2);
    /// assert_eq!(Interval::day_pow(0).unwrap(), Interval::Day);
    /// assert_eq!(Interval::day_pow(47).unwrap(), Interval::Whole);
    /// assert!(Interval::day_pow(48).is_err());
    /// ```
    pub fn day_pow(k: u8) -> Result<Interval, Error> {
        match k {
            0 => Ok(Interval::Day),
            1..=46 => Ok(Interval::DayPow { k }),
            Self::WHOLE_POW => Ok(Interval::Whole),
            _ => Err(SpatialIdError::TDayPowOutOfRange { k }.into()),
        }
    }

    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => Self::WHOLE_SECONDS,
            Interval::DayPow { k } => 86400 << k,
            Interval::Day => 86400,
            Interval::Hour => 3600,
            Interval::Minute => 60,
            Interval::Second => 1,
        }
    }

    /// 約数鎖を粗い→細かい順に列挙する（[`Whole`](Self::Whole) を含む）。
    ///
    /// 範囲分解（[`TemporalId::from_range`](crate::TemporalId::from_range)）などで使う。
    pub fn coarse_to_fine() -> impl Iterator<Item = Interval> {
        core::iter::once(Interval::Whole)
            .chain((1..Self::WHOLE_POW).rev().map(|k| Interval::DayPow { k }))
            .chain([
                Interval::Day,
                Interval::Hour,
                Interval::Minute,
                Interval::Second,
            ])
    }
}

/// 粗い方が「小さい」全順序（`Whole < … < Day < Hour < Minute < Second`）。
impl Ord for Interval {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // 秒数が大きい（粗い）ほど小さい。
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

impl TryFrom<i32> for Interval {
    type Error = Error;

    fn try_from(seconds: i32) -> Result<Self, Self::Error> {
        if seconds < 0 {
            return Err(SpatialIdError::TIntervalError { i: seconds as u64 }.into());
        }
        Self::try_from(seconds as u64)
    }
}
