//! `temporal_id` feature 無効時の [`TemporalId`] trait 実装スタブ。
//!
//! 有効時（[`impls.rs`](super::super::impls)）と同じ公開 API を保持する。

use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Display;
use core::ops::{BitAnd, Sub};
use core::str::FromStr;

use crate::{TemporalId, error::Error};

impl Display for TemporalId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/0", crate::Interval::WHOLE_SECONDS)
    }
}

impl Default for TemporalId {
    fn default() -> Self {
        Self::WHOLE
    }
}

/// 文字列表現から [`TemporalId`] を復元する。
///
/// `temporal_id` feature 無効時は任意の `"i/t"` 形式の文字列を受け付け、
/// 常に `WHOLE` を返す。
impl FromStr for TemporalId {
    type Err = Error;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self::WHOLE)
    }
}

impl BitAnd for TemporalId {
    type Output = Option<TemporalId>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl Sub for TemporalId {
    type Output = Vec<TemporalId>;
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs).collect()
    }
}
