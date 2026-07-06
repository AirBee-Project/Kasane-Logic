//! [`Interval`]: 時間IDの間隔 `i` を表す型（空間の [`ZoomLevel`](crate::ZoomLevel) に相当）。
//!
//! 生の `u64` 秒ではなく、**約数鎖の階層**として型で表す。各段は親を割り切るので、
//! 区間は必ず入れ子か非交差になる。
//!
//! 鎖は2層構造をもつ:
//! - **カレンダー層**（Day より細かい側）: day=24×hour, hour=60×min, min=60×sec。
//!   人間の慣例単位（時/分/秒/日）をそのまま保つ。
//! - **二進層**（Day より粗い側）: `Day·2^k`（`k = 1..=46`、[`day_pow`](Interval::day_pow)）。
//!   空間ズームレベルと同じ幾何スケールで、最上段 `Day·2^47` が全時間
//!   （[`Whole`](Interval::Whole)）に一致する。
//!
//! この構造により、時間ドメイン `[0, Day·2^47)` 内の**任意の区間**が高々数百個の
//! セルへ分解できる（各段で高々「分岐数−1」個 × 両端）。「全時間 − 有限セル」の
//! ような巨大な残余も対数個のセルで正確に表現できる。
//!
//! [`DayPow`](Interval::DayPow) バリアントは `#[non_exhaustive]` で、外部からは
//! 検証付きコンストラクタ（[`new`](Interval::new) / [`day_pow`](Interval::day_pow)）
//! 経由でのみ構築できる。したがって
//! **不正な指数を持つ `Interval` は型として存在しない**（パターンマッチは可能）。
//!
//! 粗い→細かい（`Whole < DayPow{46} < … < DayPow{1} < Day < Hour < Minute < Second`）の
//! 全順序を持つ。

use crate::{SpatialIdError, error::Error};

/// 全時間（[`Interval::Whole`]）の秒数 = 時間ドメインの排他的終端。
///
/// `86400 × 2^47`（約3,850億年）。UNIX秒 `[0, WHOLE_SECONDS)` が本ライブラリの
/// 時間ドメインである。
pub const WHOLE_SECONDS: u64 = 86400 << 47;

/// 二進層の最大指数（`Day·2^47` = [`Whole`](Interval::Whole)）。
const WHOLE_POW: u8 = 47;

/// 時間IDの間隔（カレンダー＋二進スケールの約数鎖）。
///
/// | バリアント | 秒数 |
/// |---|---|
/// | [`Whole`](Self::Whole) | `86400·2^47`（全時間） |
/// | [`DayPow { k }`](Self::DayPow)（k=1..=46） | `86400·2^k` |
/// | [`Day`](Self::Day) | 86400 |
/// | [`Hour`](Self::Hour) | 3600 |
/// | [`Minute`](Self::Minute) | 60 |
/// | [`Second`](Self::Second) | 1 |
///
/// `DayPow` は指数に制約（`1..=46`）があるため外部から直接構築できない。
/// [`day_pow`](Self::day_pow) または [`new`](Self::new) を使うこと。
/// 読み取りは通常どおりパターンマッチできる（`Interval::DayPow { k, .. } => …`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    /// 全時間（`86400 × 2^47` 秒 = 時間ドメイン全体）。
    Whole,
    /// `86400 × 2^k` 秒（`k = 1..=46`）。日の二進倍スケール。
    ///
    /// 検証付きコンストラクタ（[`day_pow`](Interval::day_pow) など）からのみ構築できる。
    #[non_exhaustive]
    DayPow {
        /// 指数（`1..=46`）。
        k: u8,
    },
    /// 1日（86400 秒）。
    Day,
    /// 1時間（3600 秒）。
    Hour,
    /// 1分（60 秒）。
    Minute,
    /// 1秒（1 秒）。最細。
    Second,
}

impl Interval {
    /// 秒数から[Interval]型を作成する。
    ///
    /// 約数鎖に無い秒数は
    /// [`SpatialIdError::TIntervalError`](crate::SpatialIdError::TIntervalError) を返す。
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
        match seconds {
            WHOLE_SECONDS => Ok(Interval::Whole),
            86400 => Ok(Interval::Day),
            3600 => Ok(Interval::Hour),
            60 => Ok(Interval::Minute),
            1 => Ok(Interval::Second),
            _ => {
                if seconds > 86400 && seconds.is_multiple_of(86400) {
                    let m = seconds / 86400;
                    if m.is_power_of_two() {
                        let k = m.trailing_zeros() as u8;
                        if k < WHOLE_POW {
                            return Ok(Interval::DayPow { k });
                        }
                    }
                }
                Err(SpatialIdError::TIntervalError { i: seconds }.into())
            }
        }
    }

    /// 二進層の間隔 `Day·2^k` を構築する（検証付き）。
    ///
    /// `k = 0` は [`Day`](Self::Day)、`k = 47` は [`Whole`](Self::Whole) に一致する。
    /// `k > 47` は
    /// [`SpatialIdError::TDayPowOutOfRange`](crate::SpatialIdError::TDayPowOutOfRange)
    /// を返す。
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
            WHOLE_POW => Ok(Interval::Whole),
            _ => Err(SpatialIdError::TDayPowOutOfRange { k }.into()),
        }
    }

    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => WHOLE_SECONDS,
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
            .chain((1..WHOLE_POW).rev().map(|k| Interval::DayPow { k }))
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

#[cfg(test)]
mod tests {
    use super::{Interval, WHOLE_SECONDS};
    use crate::TemporalId;

    #[test]
    fn seconds_roundtrip() {
        for iv in Interval::coarse_to_fine() {
            assert_eq!(Interval::new(iv.seconds()).unwrap(), iv, "{iv:?}");
        }
        // u64::MAX は約数鎖に無い（Whole の別名扱いはしない）
        assert!(Interval::new(u64::MAX).is_err());
        assert!(Interval::new(7200).is_err()); // 約数鎖に無い
        assert!(Interval::new(86400 * 3).is_err()); // 2冪でない日倍数
        assert!(Interval::new(0).is_err());
    }

    #[test]
    fn day_pow_validated() {
        assert_eq!(Interval::day_pow(0).unwrap(), Interval::Day);
        assert_eq!(Interval::day_pow(1).unwrap().seconds(), 86400 * 2);
        assert_eq!(Interval::day_pow(47).unwrap(), Interval::Whole);
        // 範囲外は構築できない（不正な指数を持つ Interval は存在しない）
        assert!(Interval::day_pow(48).is_err());
        assert!(Interval::day_pow(u8::MAX).is_err());
        // 読み取りはパターンマッチできる
        match Interval::day_pow(3).unwrap() {
            Interval::DayPow { k, .. } => assert_eq!(k, 3),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn chain_is_divisor_chain() {
        // 隣接する段は必ず割り切れる（入れ子・非交差の根拠）。
        let chain: alloc::vec::Vec<Interval> = Interval::coarse_to_fine().collect();
        for w in chain.windows(2) {
            let (coarse, fine) = (w[0].seconds(), w[1].seconds());
            assert!(
                coarse % fine == 0 && coarse > fine,
                "not a divisor chain: {:?} -> {:?}",
                w[0],
                w[1]
            );
        }
        // 最上段はドメイン全体
        assert_eq!(chain[0], Interval::Whole);
        assert_eq!(Interval::Whole.seconds(), WHOLE_SECONDS);
    }

    #[test]
    fn ordering_coarse_to_fine() {
        assert!(Interval::Whole < Interval::day_pow(46).unwrap());
        assert!(Interval::day_pow(46).unwrap() < Interval::day_pow(1).unwrap());
        assert!(Interval::day_pow(1).unwrap() < Interval::Day);
        assert!(Interval::Whole < Interval::Day);
        assert!(Interval::Day < Interval::Hour);
        assert!(Interval::Hour < Interval::Minute);
        assert!(Interval::Minute < Interval::Second);
    }

    #[test]
    fn temporal_id_interval_accessor() {
        let id = TemporalId::new(Interval::Hour, 10).unwrap();
        assert_eq!(id.interval(), Interval::Hour);
        assert_eq!(id.i(), 3600);
        assert_eq!(id.start_unixtime(), 36000);
        // 生秒数の from_seconds とも一致
        assert_eq!(id, TemporalId::from_seconds(3600, 10).unwrap());
        // 二進層も from_seconds で構築できる
        let two_days = TemporalId::from_seconds(86400 * 2, 3).unwrap();
        assert_eq!(two_days.interval(), Interval::day_pow(1).unwrap());
        assert_eq!(two_days.start_unixtime(), 86400 * 6);
    }
}
