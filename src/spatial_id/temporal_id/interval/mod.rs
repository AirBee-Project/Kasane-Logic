// temporal_id feature 無効時は専用のスタブを使う。
#[cfg(not(feature = "temporal_id"))]
mod disabled;
#[cfg(not(feature = "temporal_id"))]
pub use disabled::Interval;

#[cfg(feature = "temporal_id")]
use crate::{Error, SpatialIdError};

#[cfg(feature = "temporal_id")]
mod impls;

#[cfg(all(test, feature = "temporal_id"))]
mod tests;

/// 時間IDの時間間隔`i`を表現する型。`i`が任意の0より大きい自然数を受け入れると、処理に不整合が生じるためパターンを限定する。
///
/// | バリアント | 秒数 |
/// |---|---|
/// | [`Whole`](Self::Whole) | `86400·2^47` |
/// | [`DayPow { k }`](Self::DayPow)（k=1..=46） | `86400·2^k` |
/// | [`Day`](Self::Day) | 86400 |
/// | [`Hour`](Self::Hour) | 3600 |
/// | [`Minute`](Self::Minute) | 60 |
/// | [`Second`](Self::Second) | 1 |
#[cfg(feature = "temporal_id")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    /// 全時間（`86400 × 2^47` 秒）
    Whole,
    /// `86400 × 2^k` 秒（`k = 1..=46`）
    #[non_exhaustive]
    DayPow { k: u8 },
    /// 1日（86400 秒）
    Day,
    /// 1時間（3600 秒）
    Hour,
    /// 1分（60 秒）
    Minute,
    /// 1秒（1 秒）
    Second,
}

#[cfg(feature = "temporal_id")]
impl Interval {
    /// このライブラリが扱える全時間の秒数。86400 × 2^47`（約3,850億年）。
    pub const WHOLE_SECONDS: u64 = 86400 << 47;

    /// 最も粗い時間区間を表す二進層の指数。
    pub const WHOLE_POW: u8 = 47;

    /// 秒数から[Interval]型を作成する。
    ///
    /// [Interval]に当てはまらない場合は [`SpatialIdError::TIntervalError`] を返す。
    ///
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::Interval;
    /// assert_eq!(Interval::new(3600).unwrap(), Interval::Hour);
    /// assert_eq!(Interval::new(86400 * 4).unwrap(), Interval::day_pow(2).unwrap());
    /// assert!(Interval::new(7200).is_err()); // 候補に無い
    /// ```
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        Interval::try_from(seconds)
    }

    /// `Day·2^k` を作成する。
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

    /// この[Interval]の秒数。
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

    /// [Interval]の候補を粗い→細かい順に列挙する。
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
