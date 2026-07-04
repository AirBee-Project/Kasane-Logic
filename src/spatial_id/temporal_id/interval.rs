//! [`Interval`]: 時間IDの間隔 `i` を表す型（空間の [`ZoomLevel`](crate::ZoomLevel) に相当）。
//!
//! 生の `u64` 秒ではなく、**約数鎖の階層**として型で表す。各段は親を割り切るので、
//! 区間は必ず入れ子か非交差になる。
//!
//! 鎖は2層構造をもつ:
//! - **カレンダー層**（Day より細かい側）: day=24×hour, hour=60×min, min=60×sec。
//!   人間の慣例単位（時/分/秒/日）をそのまま保つ。
//! - **二進層**（Day より粗い側）: `Day·2^k`（`k = 1..=46`）。空間ズームレベルと同じ
//!   幾何スケールで、最上段 `Day·2^47` が全時間（[`Whole`](Interval::Whole)）に一致する。
//!
//! この構造により、時間ドメイン `[0, Day·2^47)` 内の**任意の区間**が高々数百個の
//! セルへ分解できる（各段で高々「分岐数−1」個 × 両端）。「全時間 − 有限セル」の
//! ような巨大な残余も対数個のセルで正確に表現できる。
//!
//! 粗い→細かい（`Whole < DayPow(46) < … < DayPow(1) < Day < Hour < Minute < Second`）の
//! 順序を持つ。

/// 全時間（[`Interval::Whole`]）の秒数 = 時間ドメインの排他的終端。
///
/// `86400 × 2^47`（約3,850億年）。UNIX秒 `[0, WHOLE_SECONDS)` が本ライブラリの
/// 時間ドメインである。
pub const WHOLE_SECONDS: u64 = 86400 << 47;

/// 二進層の最大指数（`Day·2^47` = [`Whole`](Interval::Whole)）。
const WHOLE_POW: u8 = 47;

/// 時間IDの間隔（カレンダー＋二進スケールの約数鎖）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    /// 全時間（`86400 × 2^47` 秒 = 時間ドメイン全体）。
    Whole,
    /// `86400 × 2^k` 秒（`k = 1..=46`）。日の二進倍スケール。
    DayPow(u8),
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
    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => WHOLE_SECONDS,
            Interval::DayPow(k) => 86400 << k,
            Interval::Day => 86400,
            Interval::Hour => 3600,
            Interval::Minute => 60,
            Interval::Second => 1,
        }
    }

    /// 秒数から間隔を復元する。約数鎖に無い値は `None`。
    ///
    /// 後方互換のため `u64::MAX` は [`Whole`](Interval::Whole) の別名として受け付ける。
    pub const fn from_seconds(i: u64) -> Option<Interval> {
        match i {
            u64::MAX | WHOLE_SECONDS => Some(Interval::Whole),
            86400 => Some(Interval::Day),
            3600 => Some(Interval::Hour),
            60 => Some(Interval::Minute),
            1 => Some(Interval::Second),
            _ => {
                if i > 86400 && i.is_multiple_of(86400) {
                    let m = i / 86400;
                    if m.is_power_of_two() {
                        let k = m.trailing_zeros() as u8;
                        if k < WHOLE_POW {
                            return Some(Interval::DayPow(k));
                        }
                    }
                }
                None
            }
        }
    }

    /// 約数鎖を粗い→細かい順に列挙する（[`Whole`](Interval::Whole) を含む）。
    ///
    /// 範囲分解（[`TemporalId::from_range`](crate::TemporalId::from_range)）などで使う。
    pub fn coarse_to_fine() -> impl Iterator<Item = Interval> {
        core::iter::once(Interval::Whole)
            .chain((1..WHOLE_POW).rev().map(Interval::DayPow))
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
            assert_eq!(Interval::from_seconds(iv.seconds()), Some(iv), "{iv:?}");
        }
        // 後方互換: u64::MAX は Whole の別名
        assert_eq!(Interval::from_seconds(u64::MAX), Some(Interval::Whole));
        assert_eq!(Interval::from_seconds(7200), None); // 約数鎖に無い
        assert_eq!(Interval::from_seconds(86400 * 3), None); // 2冪でない日倍数
        assert_eq!(Interval::from_seconds(0), None);
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
        assert!(Interval::Whole < Interval::DayPow(46));
        assert!(Interval::DayPow(46) < Interval::DayPow(1));
        assert!(Interval::DayPow(1) < Interval::Day);
        assert!(Interval::Whole < Interval::Day);
        assert!(Interval::Day < Interval::Hour);
        assert!(Interval::Hour < Interval::Minute);
        assert!(Interval::Minute < Interval::Second);
    }

    #[test]
    fn temporal_id_interval_accessor() {
        let id = TemporalId::from_interval(Interval::Hour, 10).unwrap();
        assert_eq!(id.interval(), Interval::Hour);
        assert_eq!(id.i(), 3600);
        assert_eq!(id.start_unixtime(), 36000);
        // 生の new とも一致
        assert_eq!(id, TemporalId::new(3600, 10).unwrap());
        // 二進層も new で構築できる
        let two_days = TemporalId::new(86400 * 2, 3).unwrap();
        assert_eq!(two_days.interval(), Interval::DayPow(1));
        assert_eq!(two_days.start_unixtime(), 86400 * 6);
    }
}
