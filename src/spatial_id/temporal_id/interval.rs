//! [`Interval`]: 時間IDの間隔 `i` を表す型（空間の [`ZoomLevel`](crate::ZoomLevel) に相当）。
//!
//! 生の `u64` 秒ではなく、**カレンダー約数鎖の階層**として型で表す。各段は親を割り切る
//! （day=24×hour, hour=60×min, min=60×sec）ので、区間は必ず入れ子か非交差になる。
//!
//! 粗い→細かい（`Whole < Day < Hour < Minute < Second`）の順序を持つ。

/// 時間IDの間隔（カレンダー階層）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    /// 全時間（間隔 `u64::MAX`）。
    Whole,
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
    /// 粗い順の finite 段（`Whole` を除く）。範囲分解などで使う。
    pub const FINITE: [Interval; 4] = [
        Interval::Day,
        Interval::Hour,
        Interval::Minute,
        Interval::Second,
    ];

    /// この間隔の秒数。`Whole` は `u64::MAX`。
    pub const fn seconds(self) -> u64 {
        match self {
            Interval::Whole => u64::MAX,
            Interval::Day => 86400,
            Interval::Hour => 3600,
            Interval::Minute => 60,
            Interval::Second => 1,
        }
    }

    /// 秒数から間隔を復元する。約数鎖に無い値は `None`。
    pub const fn from_seconds(i: u64) -> Option<Interval> {
        match i {
            u64::MAX => Some(Interval::Whole),
            86400 => Some(Interval::Day),
            3600 => Some(Interval::Hour),
            60 => Some(Interval::Minute),
            1 => Some(Interval::Second),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Interval;
    use crate::TemporalId;

    #[test]
    fn seconds_roundtrip() {
        for iv in [
            Interval::Whole,
            Interval::Day,
            Interval::Hour,
            Interval::Minute,
            Interval::Second,
        ] {
            assert_eq!(Interval::from_seconds(iv.seconds()), Some(iv));
        }
        assert_eq!(Interval::from_seconds(7200), None); // 約数鎖に無い
    }

    #[test]
    fn ordering_coarse_to_fine() {
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
        assert_eq!(id.start_unixstamp(), 36000);
        // 生の new とも一致
        assert_eq!(id, TemporalId::new(3600, 10).unwrap());
    }
}
