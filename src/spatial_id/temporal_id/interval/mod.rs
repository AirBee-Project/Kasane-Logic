/// 時間IDの時間間隔`i`を表現する型。
///
/// | バリアント | 秒数 |
/// |---|---|
/// | [`Whole`](Self::Whole) | `86400·2^47` |
/// | [`DayPow { k }`](Self::DayPow)（k=1..=46） | `86400·2^k` |
/// | [`Day`](Self::Day) | 86400 |
/// | [`Hour`](Self::Hour) | 3600 |
/// | [`Minute`](Self::Minute) | 60 |
/// | [`Second`](Self::Second) | 1 |
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

mod impls;

#[cfg(test)]
mod tests;
