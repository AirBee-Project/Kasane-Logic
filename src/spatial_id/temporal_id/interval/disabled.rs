//! `temporal_id` feature 無効時の [`Interval`] スタブ。
//!
//! 有効時（[`mod.rs`](super::mod)）と同じ公開 API を保持しつつ、
//! 常に「全時間（WHOLE）」だけを扱うシングルトン実装を提供する。

use crate::error::Error;

/// 時間IDの時間間隔を表す型（`temporal_id` feature 無効時のスタブ）。
///
/// feature 無効時は全時間（`WHOLE`）のみが有効な値であり、
/// 他の間隔を生成しようとするとエラーになる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Interval {
    #[default]
    Whole,
}

impl Interval {
    /// このライブラリが扱える全時間の秒数。`86400 × 2^47`（約3,850億年）。
    pub const WHOLE_SECONDS: u64 = 86400 << 47;

    /// 最も粗い時間区間を表す二進層の指数。
    pub const WHOLE_POW: u8 = 47;

    /// 秒数から [`Interval`] を作成する。
    ///
    /// `temporal_id` feature 無効時は [`WHOLE_SECONDS`](Self::WHOLE_SECONDS) のみ受け付ける。
    pub fn new(seconds: u64) -> Result<Interval, Error> {
        if seconds == Self::WHOLE_SECONDS {
            Ok(Interval::Whole)
        } else {
            Err(crate::SpatialIdError::TIntervalError { i: seconds }.into())
        }
    }

    /// `Day·2^k` を作成する。
    ///
    /// `temporal_id` feature 無効時は `k == WHOLE_POW` のみ受け付ける。
    pub fn day_pow(k: u8) -> Result<Interval, Error> {
        if k == Self::WHOLE_POW {
            Ok(Interval::Whole)
        } else {
            Err(crate::SpatialIdError::TIntervalError { i: k as u64 }.into())
        }
    }

    /// この間隔の秒数。
    pub const fn seconds(self) -> u64 {
        Self::WHOLE_SECONDS
    }
}

impl Ord for Interval {
    fn cmp(&self, _other: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
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
        Self::new(seconds)
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

impl_try_from_unsigned!(u8, u16, u32, u128, usize);
